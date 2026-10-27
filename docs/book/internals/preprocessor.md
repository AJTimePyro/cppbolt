# Preprocessor Internals

The preprocessor is the entry stage of the compilation pipeline, implemented in `src/preprocessor/`. It transforms raw C++ source text into an expanded translation unit while preserving accurate line origin metadata.

---

## 1. Role in the Pipeline

1. **Backslash-Newline Splicing**: Splices lines ending in `\` with subsequent lines prior to tokenization.
2. **Include Directive Processing**: Resolves `#include <...>` and `#include "..."` using an `IncludeProvider` abstraction without performing direct filesystem operations in the core library.
3. **Macro Expansion**: Expands object-like and function-like `#define` macros using standard painted-blue hide sets, macro argument expansion, token concatenation (`##`), stringification (`#`), and recursive replacement rescanning.
4. **Conditional Compilation**: Parses and evaluates constant integer expressions in `#if`, `#elif`, `#ifdef`, `#ifndef`, `#else`, and `#endif` blocks, including `defined(NAME)` queries.
5. **Origin Line Tracking**: Records structured line origin mappings (`PreprocessedSource`) so subsequent compiler errors from included headers accurately report the header file path and original line number.

---

## 2. Key Data Structures

- `Preprocessor`: The main preprocessor driver holding macro symbol tables, active condition stacks, include depth counters, and the current `IncludeProvider`.
- `PreprocessedSource`: Output struct holding the expanded source string (`.text`) and the line origin metadata table (`.origins: Vec<LineOrigin>`).
- `LineOrigin`: Represents the origin of a preprocessed line, mapping each output line back to its original file and line number.
- `MacroDef`: Stores macro definitions:
  ```rust
  pub enum MacroDef {
      Object(Vec<Token>),
      Function { params: Vec<String>, body: Vec<Token> },
  }
  ```
- `HideSet`: Set of macro names currently disabled during expansion to prevent infinite mutual recursion (standard C++ "painted-blue" algorithm).

---

## 3. Error Handling

The preprocessor returns `Result<PreprocessedSource, CompileError>`:
- **Include Cycles**: Detected by tracking active file paths on the include stack; returns `include cycle detected: '<path>'` (`tests/negative/include_cycle.cpp`).
- **Include Depth Exhaustion**: Enforces a strict limit of 128 nested include files to prevent call stack overflow.
- **Unclosed Conditionals**: Unterminated `#if` blocks at EOF trigger `Unterminated #if/#ifdef/#ifndef block at EOF`.
- **Stray Conditionals**: `#endif without #if` or `#else without #if`.
- **Diagnostic Directives**: `#error <msg>` immediately stops preprocessing with an error diagnostic; `#warning <msg>` emits a diagnostic to stderr without halting compilation.

---

## 4. Real Input & Output Example

Taken from `tests/c_basics/macro_rescan.cpp` and `tests/c_basics/preprocessor_macros.cpp`:

### Input C++ Source
```cpp
#define STRINGIFY(x) #x
#define CONCAT(a, b) a ## b

#define VALUE 42
#define GET_VAL() VALUE

int CONCAT(test_, var) = GET_VAL();
const char* name = STRINGIFY(CONCAT(hello, world));
```

### Preprocessed Output
```cpp
int test_var = 42;
const char* name = "CONCAT(hello, world)";
```

The preprocessed text is emitted alongside line origin tables so any subsequent syntax or semantic error within `test_var` points to line 7 of the source file.
