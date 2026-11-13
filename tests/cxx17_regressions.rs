//! Regression tests for C++17 member structured bindings, template array decay,
//! and member range-based for loops.

use cppbolt::{IncludeProvider, ResolvedInclude, compile_with_provider};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
struct RepoIncludes {
    stdlib: PathBuf,
}

impl IncludeProvider for RepoIncludes {
    fn read_include(&self, name: &str) -> Option<String> {
        fs::read_to_string(name).ok()
    }

    fn resolve_include(
        &self,
        name: &str,
        including_file: Option<&str>,
        quoted: bool,
    ) -> Option<ResolvedInclude> {
        let mut candidates = Vec::new();
        if quoted
            && let Some(including) = including_file
            && let Some(parent) = Path::new(including).parent()
            && !parent.as_os_str().is_empty()
        {
            candidates.push(parent.join(name));
        }
        candidates.push(self.stdlib.join(name));
        candidates.push(PathBuf::from(name));
        for path in candidates {
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(ResolvedInclude {
                    name: path.to_string_lossy().into_owned(),
                    content,
                });
            }
        }
        None
    }
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cpp(source: &str) -> Result<(i32, String), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    let wasm = compile_with_provider(source, "<input>", 256, &includes)
        .map_err(|e| format!("Compilation error: {e}"))?;

    let id = std::process::id();
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    let wasm_path = std::env::temp_dir().join(format!("cxx17_regress_{id}_{c}.wasm"));
    fs::write(&wasm_path, wasm).map_err(|e| format!("Failed to write wasm: {e}"))?;

    let host_js = root.join("web/host.js");
    let script = format!(
        r#"
const fs = require("fs");
const {{ createWasmHost }} = require({:?});
const bytes = fs.readFileSync({:?});
const host = createWasmHost({{
    stdin: "",
    onExit: (code) => {{
        process.stdout.write(host.getStdout());
        process.exit(code);
    }},
}});
WebAssembly.instantiate(bytes, host.imports).then(({{ instance }}) => {{
    let ret = 0;
    try {{
        ret = instance.exports.main ? instance.exports.main() : instance.exports._start();
    }} catch (e) {{
        console.error("WASM_TRAP: " + e.message);
        process.exit(134);
    }}
    process.stdout.write(host.getStdout());
    process.exit(ret);
}}).catch((e) => {{
    console.error("WASM_TRAP: " + e.message);
    process.exit(134);
}});
"#,
        host_js.to_string_lossy(),
        wasm_path.to_string_lossy()
    );

    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let output = Command::new(node)
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to run node: {e}"))?;

    let _ = fs::remove_file(&wasm_path);
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() && code != 0 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Execution failed with code {code}\nstdout: {stdout}\nstderr: {stderr}"
        ));
    }
    Ok((code, stdout))
}

#[test]
fn issue1_reproducer_structured_binding_in_member_function() -> Result<(), String> {
    let source = r#"
struct Pair {
    int first;
    int second;
};

struct Test {
    int sum() {
        Pair p{10, 20};
        auto [a, b] = p;
        return a + b;
    }
};

int main() {
    Test t;
    return t.sum() == 30 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "Test::sum() must return 30");
    Ok(())
}

#[test]
fn issue1_structured_binding_by_reference_in_member_function() -> Result<(), String> {
    let source = r#"
struct Pair {
    int first;
    int second;
};

struct Test {
    int mutate_and_sum() {
        Pair p{10, 20};
        auto& [a, b] = p;
        a += 5;
        b += 15;
        const auto& [ca, cb] = p;
        return ca + cb;
    }
};

int main() {
    Test t;
    return t.mutate_and_sum() == 50 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "Test::mutate_and_sum() must return 50");
    Ok(())
}

#[test]
fn issue1_structured_binding_free_function_baseline() -> Result<(), String> {
    let source = r#"
struct Pair {
    int first;
    int second;
};

int free_sum() {
    Pair p{10, 20};
    auto [a, b] = p;
    return a + b;
}

int main() {
    return free_sum() == 30 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "free_sum() must return 30");
    Ok(())
}

#[test]
fn issue2_reproducer_std_sort_array_decay() -> Result<(), String> {
    let source = r#"
#include <algorithm>

int main() {
    int values[] = {5, 2, 4, 1, 3};
    std::sort(values, values + 5);
    return values[0] == 1 && values[4] == 5 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "std::sort must sort array correctly");
    Ok(())
}

#[test]
fn issue2_user_defined_template_iterator_decay() -> Result<(), String> {
    let source = r#"
template <typename Iter>
int count_elements(Iter first, Iter last) {
    int count = 0;
    while (first != last) {
        ++count;
        ++first;
    }
    return count;
}

int main() {
    int arr[] = {10, 20, 30, 40};
    int c1 = count_elements(arr, arr + 4);
    int* ptr = arr;
    int c2 = count_elements(ptr, ptr + 4);
    return (c1 == 4 && c2 == 4) ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "count_elements must deduce int* for both array and pointer"
    );
    Ok(())
}

#[test]
fn issue2_template_reference_parameter_retains_array_type() -> Result<(), String> {
    let source = r#"
template <typename T>
bool is_same_size(const T& a, const T& b) {
    return sizeof(a) == sizeof(b);
}

int main() {
    int arr1[5];
    int arr2[5];
    return is_same_size(arr1, arr2) ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "const T& parameter must retain array type in deduction"
    );
    Ok(())
}

#[test]
fn issue3_reproducer_range_for_in_member_function_nsdmi() -> Result<(), String> {
    let source = r#"
struct Container {
    int values[3] = {10, 20, 30};

    int sum() {
        int result = 0;

        for (int value : values) {
            result += value;
        }

        return result;
    }
};

int main() {
    Container c;
    return c.sum() == 60 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "Container::sum() with NSDMI array must return 60");
    Ok(())
}

#[test]
fn issue3_range_for_in_member_function_ctor_init() -> Result<(), String> {
    let source = r#"
struct Container {
    int values[3];

    Container() {
        values[0] = 10;
        values[1] = 20;
        values[2] = 30;
    }

    int sum() {
        int result = 0;

        for (int value : values) {
            result += value;
        }

        return result;
    }
};

int main() {
    Container c;
    return c.sum() == 60 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "Container::sum() with ctor init must return 60");
    Ok(())
}

#[test]
fn issue3_range_for_in_member_function_container_iterators() -> Result<(), String> {
    let source = r#"
#include <vector>

struct Processor {
    std::vector<int> items;

    int total() {
        int sum = 0;
        for (int x : items) {
            sum += x;
        }
        return sum;
    }
};

int main() {
    Processor p;
    p.items.push_back(10);
    p.items.push_back(20);
    p.items.push_back(30);
    return p.total() == 60 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "Processor::total() over std::vector must return 60"
    );
    Ok(())
}

#[test]
fn issue3_range_for_free_function_baseline() -> Result<(), String> {
    let source = r#"
int sum_array() {
    int values[3] = {10, 20, 30};
    int result = 0;
    for (int v : values) {
        result += v;
    }
    return result;
}

int main() {
    return sum_array() == 60 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "sum_array() free function must return 60");
    Ok(())
}

#[test]
fn member_function_auto_return_type_deduction() -> Result<(), String> {
    let source = r#"
struct Calculator {
    auto add(int a, int b) {
        return a + b;
    }
};

int main() {
    Calculator c;
    return c.add(15, 27) == 42 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0, "Calculator::add() with auto return must return 42");
    Ok(())
}

#[test]
fn array_member_of_structs_nested_brace_init() -> Result<(), String> {
    let source = r#"
struct Point {
    int x;
    int y;
};

struct Canvas {
    Point pts[2] = {{10, 20}, {30, 40}};
};

int main() {
    Canvas c;
    return (c.pts[0].x == 10 && c.pts[0].y == 20 && c.pts[1].x == 30 && c.pts[1].y == 40) ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "Canvas::pts array of structs with nested brace init must initialize members"
    );
    Ok(())
}

#[test]
fn include_directive_without_space_before_header() -> Result<(), String> {
    let source = r#"
#include<iostream>
using namespace std;

int findSum(int n)
{
    return n * (n + 1) / 2;
}

int main()
{
    int n = 5;
    cout << findSum(n);
    return findSum(n) == 15 ? 0 : 1;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "#include<iostream> must be accepted like #include <iostream>"
    );
    assert_eq!(stdout, "15");
    Ok(())
}

#[test]
fn range_for_const_reference_inside_function_template() -> Result<(), String> {
    let source = r#"
#include <array>

template <typename T, int N>
int sum_all(const std::array<T, N>& values) {
    int sum = 0;
    for (const T& value : values) {
        sum += value;
    }
    return sum;
}

int main() {
    std::array<int, 3> values = {10, 20, 30};
    return sum_all(values) == 60 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "const T& range-for must be substituted inside a function template"
    );
    Ok(())
}

#[test]
fn range_for_vector_inside_function_template() -> Result<(), String> {
    let source = r#"
#include <vector>

template <typename T>
int sum_all(const std::vector<T>& values) {
    int sum = 0;
    for (const T& value : values) {
        sum += value;
    }
    return sum;
}

int main() {
    std::vector<int> values;
    values.push_back(10);
    values.push_back(20);
    values.push_back(30);
    return sum_all(values) == 60 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "const T& range-for over std::vector must work in a function template"
    );
    Ok(())
}

#[test]
fn template_body_substitutes_switch_and_structured_binding() -> Result<(), String> {
    let source = r#"
struct Pair {
    int first;
    int second;
};

template <typename T>
int pick(T value) {
    auto [a, b] = Pair{value, value + 1};
    switch (b - a) {
        case 1:
            return a + b;
        default:
            return 0;
    }
}

int main() {
    return pick(20) == 41 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(
        code, 0,
        "template statement substitution must cover switch and structured bindings"
    );
    Ok(())
}
