# Supported C++ Feature Matrix

`cppwasm` implements a defined, correctness-oriented subset of C++17. Every supported feature is covered by automated corpus tests and pinned by cryptographic golden hashes. Features outside the subset are either explicitly rejected with clear diagnostics or tracked as planned future extensions.

The baseline test status is tracked in [`tests/PASSING.txt`](https://github.com/cppwasm/cpp_web/blob/main/tests/PASSING.txt). Run the test runner to verify status locally:

```sh
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

---

## Feature Matrix

| Feature | Status | Notes | Test |
|---|---|---|---|
| **Scalar Types** (`int`, `long long`, `short`, `char`, `float`, `double`, `bool`) | **Done** | ILP32 32-bit data model; unsigned overflow wrapping | [`tests/c_basics/arithmetic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/arithmetic.cpp), [`tests/c_basics/long_long.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/long_long.cpp), [`tests/c_basics/floats.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/floats.cpp) |
| **Pointers & Pointer Arithmetic** | **Done** | 32-bit Wasm `i32` addresses; array decay | [`tests/c_basics/pointers_arrays.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/pointers_arrays.cpp) |
| **Function Pointers** | **Done** | Indirect calls via WebAssembly function table; comparator callbacks | [`tests/classes_oop/function_pointer_comparator.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/function_pointer_comparator.cpp) |
| **Multi-Dimensional Arrays** | **Done** | Nested dimensions wrap inside-out; contiguous row-major layout | [`tests/c_basics/multidim_arrays.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/multidim_arrays.cpp) |
| **Lvalue References (`T&`)** | **Done** | Reference binding, parameters, reference return values | [`tests/classes_oop/references.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/references.cpp), [`tests/classes_oop/member_reference_return.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/member_reference_return.cpp) |
| **`const` Qualifiers** | **Done** | `const` variables, `const` pointers, `const` member methods | [`tests/c_basics/const_qualifiers.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/const_qualifiers.cpp), [`tests/classes_oop/const_methods.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/const_methods.cpp) |
| **`auto` Type Deduction** | **Done** | Local variable and reference type deduction | [`tests/c_basics/auto_type.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/auto_type.cpp) |
| **Type Aliases (`typedef`, `using`)** | **Done** | Standard `typedef` and C++11 `using Alias = Type;` | [`tests/c_basics/auto_type.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/auto_type.cpp) |
| **Multi-Declarator Declarations** | **Done** | Comma-separated declarators sharing base specifiers (`int *p, q = 2;`) | [`tests/c_basics/multi_declarator.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/multi_declarator.cpp) |
| **Enums** | **Done** | Scoped (`enum class`) and unscoped `enum` | [`tests/c_basics/enums.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/enums.cpp) |
| **Control Flow** (`if`, `switch`, loops) | **Done** | `if`/`else`, ternary `?:`, `while`, `for`, `do-while`, `switch`/`case`/`default` | [`tests/c_basics/control_if_else.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/control_if_else.cpp), [`tests/c_basics/control_loops.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/control_loops.cpp), [`tests/c_basics/control_switch.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/control_switch.cpp) |
| **Functions & Overloading** | **Done** | Exact match > reference binding > promotion > conversion; default arguments | [`tests/c_basics/functions_call.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/functions_call.cpp), [`tests/c_basics/functions_overload.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/functions_overload.cpp), [`tests/c_basics/overload_resolution.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/overload_resolution.cpp) |
| **Static Variables** | **Done** | Function-local statics with arguments; class static members | [`tests/classes_oop/function_local_statics.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/function_local_statics.cpp), [`tests/classes_oop/static_members.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/static_members.cpp) |
| **Structs & Classes** | **Done** | Aggregate init, member variables/methods, access control | [`tests/classes_oop/struct_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/struct_basic.cpp), [`tests/classes_oop/initializer_lists.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/initializer_lists.cpp) |
| **Constructors & Destructors** | **Done** | Default, parameterized, copy constructors; initializer list ordering | [`tests/classes_oop/copy_semantics.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/copy_semantics.cpp), [`tests/classes_oop/member_objects_order.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/member_objects_order.cpp) |
| **RAII Lifecycle & Scope Cleanups** | **Done** | Cleanups on scope exit, early `return`, `break`, `continue`, statement temporaries | [`tests/classes_oop/class_raii.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/class_raii.cpp), [`tests/classes_oop/raii_early_return.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/raii_early_return.cpp), [`tests/classes_oop/raii_temporaries.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/raii_temporaries.cpp) |
| **C++17 Copy Elision** | **Done** | Prvalue construction in place (direct init and function return) | [`tests/classes_oop/prvalue_elision.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/prvalue_elision.cpp), [`tests/classes_oop/direct_initialization_elision.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/direct_initialization_elision.cpp) |
| **Dynamic Memory (`new` / `delete`)** | **Done** | Single object and array `new[]`/`delete[]`; placement `new` | [`tests/classes_oop/new_delete.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/new_delete.cpp), [`tests/classes_oop/new_delete_object_arrays.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/new_delete_object_arrays.cpp), [`tests/classes_oop/placement_new.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/placement_new.cpp) |
| **Single & Linear Inheritance** | **Done** | Base subobjects, slicing, constructor/destructor chains | [`tests/classes_oop/single_inheritance.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/single_inheritance.cpp), [`tests/classes_oop/multi_level_inheritance.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/multi_level_inheritance.cpp) |
| **Virtual Functions & Polymorphism** | **Done** | Vtable dispatch, virtual destructors, pure virtual (`= 0`) | [`tests/classes_oop/virtual_functions.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/virtual_functions.cpp), [`tests/classes_oop/virtual_destructor.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/virtual_destructor.cpp), [`tests/classes_oop/pure_virtual.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/pure_virtual.cpp) |
| **Namespaces** | **Done** | Nested namespaces, `using namespace`, qualified lookups | [`tests/classes_oop/namespaces_comprehensive.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/namespaces_comprehensive.cpp) |
| **Operator Overloading (Binary & Compound)** | **Done** | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `<<`, `>>`, `&`, `^`, `\|`, `=`, `+=`, `-=`, `*=` | [`tests/templates_operators/op_overload_binary.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/templates_operators/op_overload_binary.cpp), [`tests/classes_oop/chained_compound_assign.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/chained_compound_assign.cpp) |
| **Operator Overloading (Prefix & Subscript)** | **Done** | Unary `+`, `-`, `!`, `~`, prefix `++`/`--`, subscript `[]`, `operator bool()` | [`tests/templates_operators/op_overload_unary.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/templates_operators/op_overload_unary.cpp), [`tests/templates_operators/op_overload_subscript.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/templates_operators/op_overload_subscript.cpp), [`tests/classes_oop/contextual_bool_comprehensive.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/classes_oop/contextual_bool_comprehensive.cpp) |
| **Preprocessor** | **Done** | `#define` (function/object, `#`, `##`), `#include`, `#if`/`#elif`, `#error`, `#warning`, `\` | [`tests/c_basics/preprocessor.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/preprocessor.cpp), [`tests/c_basics/macro_rescan.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/macro_rescan.cpp), [`tests/c_basics/warning_directive.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/warning_directive.cpp) |
| **C Standard I/O (`<cstdio>`)** | **Done** | `printf` (all conversions `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%E`, `%f`, `%g`, `%G`), `sprintf`, `snprintf`, `puts` | [`tests/c_basics/printf_conversions.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/printf_conversions.cpp), [`tests/c_basics/printf_table.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/c_basics/printf_table.cpp), [`tests/mini_stl/cstdio_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/cstdio_basic.cpp) |
| **C++ Streams (`<iostream>`)** | **Done** | `std::cout`, `std::cin`, `std::getline`, custom stream operators | [`tests/mini_stl/iostream_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/iostream_basic.cpp), [`tests/mini_stl/iostream_cin.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/iostream_cin.cpp), [`tests/mini_stl/stream_operator_overload.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/stream_operator_overload.cpp) |
| **Standard String (`<string>`)** | **Done** | `std::string`, `.c_str()`, `.length()`, `.append()`, concatenation | [`tests/mini_stl/string_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/string_basic.cpp), [`tests/mini_stl/string_comprehensive.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/string_comprehensive.cpp) |
| **Math & Formatting** (`<cmath>`, `<iomanip>`) | **Done** | Math intrinsics, `std::setprecision`, `std::fixed`, `std::setw` | [`tests/mini_stl/cmath_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/cmath_basic.cpp), [`tests/mini_stl/iostream_precision_after_fixed.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/iostream_precision_after_fixed.cpp) |
| **Templates** (`template <typename T>`) | **Planned** | Lexed but not parsed; expected failures in test runner | [`tests/templates_operators/template_class_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/templates_operators/template_class_basic.cpp), [`tests/templates_operators/template_func_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/templates_operators/template_func_basic.cpp) |
| **Standard Containers & Algorithms** (`vector`, `map`, `set`, `algorithm`) | **Planned** | Headers present in `stdlib/` but template instantiations fail | [`tests/mini_stl/vector_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/vector_basic.cpp), [`tests/mini_stl/map_basic.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/map_basic.cpp), [`tests/mini_stl/algorithm_sort.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/mini_stl/algorithm_sort.cpp) |
| **Range-Based `for` Loops** | **Planned** | Explicitly rejected by parser with `range-based for is not supported` | [`tests/negative/range_for_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/range_for_unsupported.cpp) |
| **Overloaded Postfix `++` / `--`** | **Planned** | Primitive postfix works; overloading `operator++(int)` is explicitly rejected | [`tests/negative/postfix_increment_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/postfix_increment_unsupported.cpp) |
| **Overloaded `->`, `*()`, `()`** | **Planned** | Primitive `->` and `*` work; overloading `operator->`, `operator*()`, or `operator()` is explicitly rejected | [`tests/negative/operator_arrow_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/operator_arrow_unsupported.cpp), [`tests/negative/operator_call_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/operator_call_unsupported.cpp), [`tests/negative/operator_deref_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/operator_deref_unsupported.cpp) |
| **Const-Ref Method Overloading** (`T m() const &`) | **Planned** | `const` methods supported; `const &` method ref-qualifiers are rejected with `const overloading is not supported` | [`tests/negative/const_overload_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/const_overload_unsupported.cpp) |
| **Namespace Aliases** (`namespace A = B;`) | **Planned** | Explicitly rejected by parser with `namespace aliases are not supported` | [`tests/negative/namespace_alias.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/namespace_alias.cpp) |
| **Multiple & Virtual Inheritance** | **Out of Scope** | Single/linear inheritance supported; multiple and virtual base specifiers are rejected | [`tests/negative/multiple_inheritance.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/multiple_inheritance.cpp), [`tests/negative/virtual_inheritance.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/virtual_inheritance.cpp) |
| **Exceptions & RTTI** | **Out of Scope** | Excluded in favor of deterministic, unwinding-free direct Wasm lowering ([ADR 0005](../adr/0005-no-exceptions-rtti.md)) | N/A |

---

## Known Limitations & Design Rules

### 1. Language Semantics & Edge Cases
- **Copy Elision**: Prvalue initialization is elided as guaranteed by C++17 (`T x = T(1);`). However, return of named local variables requires running the copy constructor.
- **Null Pointer Constant Tracking**: An integer literal `0` is accepted where a pointer is expected, but general constant-expression null pointer evaluation is not tracked.
- **Multiple and Virtual Inheritance**: Explicitly rejected at parse time:
  - Second base specifier yields: `multiple inheritance is not supported`
  - `virtual` base specifier yields: `virtual inheritance is not supported`
- **Operator Overloading Boundaries**: While built-in primitive pointer member access `->`, pointer dereference `*`, and numeric postfix `++`/`--` are fully supported, **user-defined operator overloads** for `operator->`, `operator*()`, `operator()`, and postfix `operator++(int)` / `operator--(int)` are rejected by the parser (`src/parser/unsupported.rs`). Method ref-qualifier overloading (`const &`) is likewise rejected.

### 2. Nesting Limits per Construct

To protect WebAssembly engine shadow stacks from exhaustion when compiling deep programs in Web Workers, the compiler enforces per-construct nesting budgets ([`src/depth.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/depth.rs)). One construct cannot exhaust the budget of another:

| Construct | Limit | Deepest Accepted Input (Native & Wasm) |
|---|---|---|
| **Expressions** | 4,000 | Parenthesized conditional costs 2 levels |
| **Parentheses** | 1,600 | 1,600 nested parentheses |
| **Call Argument Lists** | 1,500 | 1,500 nested calls |
| **Statements** | 1,600 | 1,599 nested `if` statements |
| **Compound Blocks** | 1,600 | 1,599 nested `{}` blocks |
| **Declarations** | 1,600 | 1,600 nested declarations |
| **Type Expressions** | 1,600 | 1,600 nested type qualifiers |
| **Preprocessor Conditions** | 1,600 | 1,600 nested `#if` conditions |
| **Include Depth** | 128 | 128 nested `#include` directives (129 is rejected) |

These limits are verified by [`tests/nesting.rs`](https://github.com/cppwasm/cpp_web/blob/main/tests/nesting.rs) and [`tests/wasm_stack_safety.rs`](https://github.com/cppwasm/cpp_web/blob/main/tests/wasm_stack_safety.rs).

Wide left-associative expressions (sums of 20,000 terms), string concatenations (4,000 terms), and `else-if` chains (5,000 branches) are flattened iteratively and do not consume recursion depth.

### 3. Printf Runtime Specifications
The embedded `printf` engine ([`src/runtime/format.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/runtime/format.rs)) supports:
- Specifiers: `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%E`, `%f`, `%g`, `%G`.
- Modifiers: `h`, `hh`, `l`, `ll`, and `z`.
- Flags: `0` (zero-pad), `-` (left-align), `+` (sign), ` ` (space sign), `#` (alternate form).
- Dynamic precision: `.*` is supported. (Dynamic width `*` is not implemented).
- Unsupported conversions: `%p` and `%n`.
- NaN and Infinities: Zero padding falls back to spaces; sign of NaN is implementation-defined.
