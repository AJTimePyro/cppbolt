# Supported C++ Feature Matrix

`cppbolt` implements a defined, correctness-oriented subset of C++17. Every supported feature is covered by automated corpus tests and pinned by cryptographic golden hashes. Features outside the subset are either explicitly rejected with clear diagnostics or tracked as planned future extensions.

The baseline test status is tracked in `tests/PASSING.txt`. Run the test runner to verify status locally:

```sh
cargo run --bin cppwasm-testrunner -- target/release/cppwasm-cli
```

---

## Feature Matrix

| Feature | Status | Notes | Test |
|---|---|---|---|
| **Scalar Types** (`int`, `long long`, `short`, `char`, `float`, `double`, `bool`) | **Done** | ILP32 32-bit data model; unsigned overflow wrapping | `tests/c_basics/types/arithmetic.cpp`, `tests/c_basics/types/long_long.cpp`, `tests/c_basics/types/floats.cpp` |
| **Pointers & Pointer Arithmetic** | **Done** | 32-bit Wasm `i32` addresses; array decay | `tests/c_basics/pointers_arrays/pointers_arrays.cpp` |
| **Function Pointers** | **Done** | Indirect calls via WebAssembly function table; comparator callbacks | `tests/classes_oop/methods/function_pointer_comparator.cpp` |
| **Multi-Dimensional Arrays** | **Done** | Nested dimensions wrap inside-out; contiguous row-major layout; pointer-to-array decay | `tests/c_basics/pointers_arrays/multidim_arrays.cpp`, `tests/c_basics/pointers_arrays/multidim_pointer_deref.cpp` |
| **Lvalue References (`T&`)** | **Done** | Reference binding, parameters, reference return values | `tests/classes_oop/statics_namespaces/references.cpp`, `tests/classes_oop/methods/member_reference_return.cpp` |
| **`const` Qualifiers** | **Done** | `const` variables, `const` pointers, `const` member methods | `tests/c_basics/types/const_qualifiers.cpp`, `tests/classes_oop/methods/const_methods.cpp` |
| **`auto` Type Deduction** | **Done** | Local variable and reference type deduction | `tests/c_basics/types/auto_type.cpp` |
| **Type Aliases (`typedef`, `using`)** | **Done** | Standard `typedef` and C++11 `using Alias = Type;` | `tests/c_basics/types/auto_type.cpp` |
| **Multi-Declarator Declarations** | **Done** | Comma-separated declarators sharing base specifiers (`int *p, q = 2;`) | `tests/c_basics/misc/multi_declarator.cpp` |
| **Array Function Parameters** | **Done** | Parameter adjustment decaying `int arr[]` and `int arr[N]` to `int* arr` | `tests/c_basics/pointers_arrays/array_parameter_decay.cpp` |
| **Enums** | **Done** | Scoped (`enum class`) with qualified lookup (`Scope::Val`) and unscoped `enum` | `tests/c_basics/enums/enums.cpp`, `tests/c_basics/enums/scoped_enums.cpp` |
| **Control Flow** (`if`, `switch`, loops) | **Done** | `if`/`else`, ternary `?:`, `while`, `for`, `do-while`, `switch`/`case`/`default` (with enum & const-expr `case` labels) | `tests/c_basics/control_flow/control_if_else.cpp`, `tests/c_basics/control_flow/control_loops.cpp`, `tests/c_basics/control_flow/switch_enum_case.cpp` |
| **Functions & Overloading** | **Done** | Exact match > reference binding > promotion > conversion; default arguments; global init reachability | `tests/c_basics/functions/functions_call.cpp`, `tests/c_basics/functions/constexpr_global_init_call.cpp`, `tests/c_basics/functions/overload_resolution.cpp` |
| **Static Variables** | **Done** | Function-local statics with arguments; class static members | `tests/classes_oop/statics_namespaces/function_local_statics.cpp`, `tests/classes_oop/statics_namespaces/static_members.cpp` |
| **Structs & Classes** | **Done** | Aggregate init, member variables/methods, access control | `tests/classes_oop/structs/struct_basic.cpp`, `tests/classes_oop/structs/initializer_lists.cpp` |
| **Constructors & Destructors** | **Done** | Default, parameterized, copy, delegating (`C() : C(x) {}`); initializer list ordering; implicit synthesis for trivial/empty classes and prvalues (`Value().get()`); global parenthesized construction | `tests/classes_oop/constructors_destructors/copy_semantics.cpp`, `tests/classes_oop/constructors_destructors/delegating_constructor.cpp`, `tests/classes_oop/constructors_destructors/prvalue_default_constructor.cpp`, `tests/classes_oop/statics_namespaces/global_constructor_args.cpp` |
| **Conversion Operators** | **Done** | User-defined conversion operators `operator T() const` with `explicit` qualification checks; disambiguated from copy constructors | `tests/classes_oop/operators/conversion_operator.cpp`, `tests/classes_oop/operators/regress_user_conversion_param.cpp`, `tests/classes_oop/operators/regress_byval_param_conversion.cpp` |
| **Ref-Qualified Methods** | **Done** | Lvalue and rvalue ref-qualified member functions (`get() &`, `get() &&`) with receiver value-category dispatch | `tests/classes_oop/methods/ref_qualified_methods.cpp` |
| **Structured Bindings** | **Done** | Aggregate decomposition (`auto [a, b] = p;`, `auto&`, `const auto&`) | `tests/classes_oop/structs/structured_bindings.cpp` |
| **RAII Lifecycle & Scope Cleanups** | **Done** | Cleanups on scope exit, early `return`, `break`, `continue`, statement temporaries | `tests/classes_oop/memory_raii/class_raii.cpp`, `tests/classes_oop/memory_raii/raii_early_return.cpp`, `tests/classes_oop/memory_raii/raii_temporaries.cpp` |
| **C++17 Copy Elision** | **Done** | Prvalue construction in place (direct init and function return) | `tests/classes_oop/memory_raii/prvalue_elision.cpp`, `tests/classes_oop/memory_raii/direct_initialization_elision.cpp` |
| **Dynamic Memory (`new` / `delete`)** | **Done** | Single object and array `new[]`/`delete[]`; placement `new` | `tests/classes_oop/memory_raii/new_delete.cpp`, `tests/classes_oop/memory_raii/new_delete_object_arrays.cpp`, `tests/classes_oop/memory_raii/placement_new.cpp` |
| **Single & Linear Inheritance** | **Done** | Base subobjects, slicing, constructor/destructor chains | `tests/classes_oop/inheritance/single_inheritance.cpp`, `tests/classes_oop/inheritance/multi_level_inheritance.cpp` |
| **Virtual Functions & Polymorphism** | **Done** | Vtable dispatch, virtual destructors, pure virtual (`= 0`) | `tests/classes_oop/inheritance/virtual_functions.cpp`, `tests/classes_oop/inheritance/virtual_destructor.cpp`, `tests/classes_oop/inheritance/pure_virtual.cpp` |
| **Namespaces** | **Done** | Nested namespaces, `using namespace`, qualified lookups | `tests/classes_oop/statics_namespaces/namespaces_comprehensive.cpp` |
| **Operator Overloading (Binary & Compound)** | **Done** | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `<<`, `>>`, `&`, `^`, `\|`, `=`, `+=`, `-=`, `*=` | `tests/templates_operators/op_overload_binary.cpp`, `tests/classes_oop/operators/chained_compound_assign.cpp` |
| **Operator Overloading (Prefix & Subscript)** | **Done** | Unary `+`, `-`, `!`, `~`, prefix `++`/`--`, subscript `[]`, `operator bool()` | `tests/templates_operators/op_overload_unary.cpp`, `tests/templates_operators/op_overload_subscript.cpp`, `tests/classes_oop/operators/contextual_bool_comprehensive.cpp` |
| **Preprocessor** | **Done** | `#define` (function/object, `#`, `##`), `#include`, `#if`/`#elif`, `#error`, `#warning`, `\` | `tests/c_basics/preprocessor/preprocessor.cpp`, `tests/c_basics/preprocessor/macro_rescan.cpp`, `tests/c_basics/preprocessor/warning_directive.cpp` |
| **C Standard I/O (`<cstdio>`)** | **Done** | `printf` (all conversions `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%E`, `%f`, `%g`, `%G`), `sprintf`, `snprintf`, `puts` | `tests/c_basics/io_printf/printf_conversions.cpp`, `tests/c_basics/io_printf/printf_table.cpp`, `tests/mini_stl/cstdio_basic.cpp` |
| **C++ Streams (`<iostream>`)** | **Done** | `std::cout`, `std::cin`, `std::getline`, custom stream operators | `tests/mini_stl/iostream_basic.cpp`, `tests/mini_stl/iostream_cin.cpp`, `tests/mini_stl/stream_operator_overload.cpp` |
| **Standard String (`<string>`)** | **Done** | `std::string`, `.c_str()`, `.length()`, `.append()`, concatenation | `tests/mini_stl/string_basic.cpp`, `tests/mini_stl/string_comprehensive.cpp` |
| **Math & Formatting** (`<cmath>`, `<iomanip>`) | **Done** | Math intrinsics, `std::setprecision`, `std::fixed`, `std::setw` | `tests/mini_stl/cmath_basic.cpp`, `tests/mini_stl/iostream_precision_after_fixed.cpp` |
| **Templates** (`template <typename T>`) | **Planned** | Lexed but not parsed; expected failures in test runner | `tests/templates_operators/template_class_basic.cpp`, `tests/templates_operators/template_func_basic.cpp` |
| **Standard Containers & Algorithms** (`vector`, `map`, `set`, `algorithm`) | **Planned** | Headers present in `stdlib/` but template instantiations fail | `tests/mini_stl/vector_basic.cpp`, `tests/mini_stl/map_basic.cpp`, `tests/mini_stl/algorithm_sort.cpp` |
| **Range-Based `for` Loops** | **Done** | C++11 `for (decl : range)` over fixed-size arrays (1D and multi-dimensional) and classes with `.begin()`/`.end()`; supports `auto`, `const auto`, `auto&`, `const auto&` | `tests/c_basics/pointers_arrays/range_for_array.cpp`, `tests/mini_stl/range_for_string.cpp` |
| **Overloaded Postfix `++` / `--`** | **Done** | Overloaded `operator++(int)` and `operator--(int)` distinguish postfix from prefix via dummy `int` parameter | `tests/classes_oop/operators/operator_postfix_incdec.cpp` |
| **Overloaded `->`, `*()`, `()`** | **Done** | Smart-pointer and functor operators: `operator->` (recursive chaining with cycle detection), dereference `operator*()`, and function call `operator()` | `tests/classes_oop/operators/operator_smart_pointer.cpp`, `tests/classes_oop/operators/operator_arrow_chaining.cpp`, `tests/classes_oop/operators/operator_call_functor.cpp` |
| **Lambda Expressions (C++17)** | **Done** | `[]`, `[x]`, `[&x]`, `[=]`, `[&]`, `mutable`, trailing return type (`-> T`), return type deduction, `auto` storage, direct invocation, unary `+` decay, and implicit conversion of captureless lambdas to C function pointers (`R (*)(P...)`) including return targets | `tests/classes_oop/lambdas/lambda_basic.cpp`, `tests/classes_oop/lambdas/lambda_default_captures.cpp`, `tests/classes_oop/lambdas/lambda_function_pointer.cpp`, `tests/classes_oop/lambdas/lambda_return_fn_ptr.cpp`, `tests/classes_oop/lambdas/lambda_range_for_capture.cpp`, `tests/classes_oop/lambdas/lambda_unary_plus.cpp` |
| **List Initialization & Aggregates** | **Done** | Direct-list (`T x{...}`), copy-list (`T x = {...}`), empty brace value initialization (`{}`), aggregate base subobjects, brace elision, narrowing conversion checks ([dcl.init.list] p7), dynamic allocation (`new T{...}`, `new T[n]{...}`), typed temporaries (`Type{...}.get()`), and explicit constructor enforcement | `tests/classes_oop/structs/brace_init_scalars.cpp`, `tests/classes_oop/structs/brace_init_classes.cpp`, `tests/classes_oop/structs/brace_init_aggregates.cpp`, `tests/classes_oop/structs/brace_init_brace_elision.cpp`, `tests/classes_oop/structs/brace_init_temporaries.cpp`, `tests/classes_oop/structs/brace_init_dynamic_new.cpp` |
| **Const-Ref Method Overloading** (`T m() const &`) | **Planned** | `const` methods supported; `const &` method ref-qualifiers are rejected with `const overloading is not supported` | `tests/negative/const_overload_unsupported.cpp` |
| **Namespace Aliases** (`namespace A = B;`) | **Planned** | Explicitly rejected by parser with `namespace aliases are not supported` | `tests/negative/namespace_alias.cpp` |
| **Multiple & Virtual Inheritance** | **Out of Scope** | Single/linear inheritance supported; multiple and virtual base specifiers are rejected | `tests/negative/multiple_inheritance.cpp`, `tests/negative/virtual_inheritance.cpp` |
| **Exceptions (`throw`)** | **Diagnosed** | `throw` keyword recognized and rejected with explicit target diagnostic (`C++ exception handling ('throw') is not supported on this WebAssembly target`) | `tests/negative/throw_unsupported.cpp` |

---

## Known Limitations & Design Rules

### 1. Language Semantics & Edge Cases
- **Copy Elision**: Prvalue initialization is elided as guaranteed by C++17 (`T x = T(1);`). However, return of named local variables requires running the copy constructor.
- **Null Pointer Constant Tracking**: An integer literal `0` is accepted where a pointer is expected, but general constant-expression null pointer evaluation is not tracked.
- **Multiple and Virtual Inheritance**: Explicitly rejected at parse time:
  - Second base specifier yields: `multiple inheritance is not supported`
  - `virtual` base specifier yields: `virtual inheritance is not supported`
- **Operator Overloading Boundaries**: User-defined operator overloads for arithmetic, relational, bitwise, assignment, subscript `[]`, smart-pointer `->`, dereference `*()`, functor `()`, postfix `++`/`--`, and conversion operators `operator T()` are supported. Method ref-qualifier overloading with const-lvalue references (`const &`) is explicitly rejected with `const overloading is not supported`.
- **Chained `operator->` & Cycle Detection**: In accordance with C++17, member access via `->` recursively resolves through chained `operator->()` overloads returning class objects or references until a raw struct pointer is reached. Cycle detection prevents compile-time recursion loops (`circular operator-> delegation on '<class>'`), and recursion depth is budgeted to 128 hops.
- **C++17 Lambda Expressions**: Lambdas desugar into anonymous closure structs (`__lambda_N`) holding captured state as either by-value copies or references. The synthesized `operator()` is `const`-qualified unless marked `mutable`, enforcing immutability on by-value captures. Return types can be explicitly annotated with trailing return syntax (`-> T`) or automatically deduced from `return` statements. Captureless lambdas (`[]`) synthesize a static `__invoke` trampoline enabling implicit conversion to standard C function pointers (`R (*)(P...)`), including as return statement targets and via unary `+` decay (`+[](){}`). Default captures (`=`, `&`) must strictly appear first in the capture list, and free-variable discovery traverses `RangeFor`, `StructuredBinding`, and nested `Lambda`/`New` constructs.
- **List Initialization & Aggregates**: Braced initializer lists (`{...}`) work across variable declarations, constructor member initializers (`: member{val}`), returns, call arguments, assignments, typed temporaries (`Type{args...}.member()`), and heap allocations (`new T{...}`, `new T[n]{...}`).
  - Direct-list initialization allows selecting `explicit` constructors; copy-list initialization rejects them. Empty braces (`{}`) value-initialize scalars and enums to zero, clear aggregate fields, or call default constructors.
  - Under C++17 aggregate rules, classes qualify as aggregates when they have no user-declared constructors, no private or protected non-static data members, no virtual functions, and no virtual or private base classes. Public non-virtual bases initialize as initial subobjects, either with nested braces (`Derived{{base_args}, derived_args}`) or flattened across the outer list.
  - Narrowing conversion checks follow C++17 [dcl.init.list] p7, rejecting conversions from floating-point to integer, `double` to `float` (unless an exact constant literal), and between integer types with sign changes or smaller ranges. Compile-time constant expressions, including negative integer literals, are accepted when the value fits the destination type.
  - Single-object `new T{...}` direct-initializes the heap object. Array `new T[n]{...}` initializes explicit elements and runs a WebAssembly loop to value-initialize omitted trailing elements up to `n`.
  - Initializer list constructors taking `std::initializer_list<T>` require template engine support and report an explicit diagnostic.

### 2. Nesting Limits per Construct

To protect WebAssembly engine shadow stacks from exhaustion when compiling deep programs in Web Workers, the compiler enforces per-construct nesting budgets (`src/depth.rs`). One construct cannot exhaust the budget of another:

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

These limits are verified by `tests/nesting.rs` and `tests/wasm_stack_safety.rs`.

Wide left-associative expressions (sums of 20,000 terms), string concatenations (4,000 terms), and `else-if` chains (5,000 branches) are flattened iteratively and do not consume recursion depth.

### 3. Printf Runtime Specifications
The embedded `printf` engine (`src/runtime/format.rs`) supports:
- Specifiers: `%d`, `%i`, `%u`, `%o`, `%x`, `%X`, `%c`, `%s`, `%e`, `%E`, `%f`, `%g`, `%G`.
- Modifiers: `h`, `hh`, `l`, `ll`, and `z`.
- Flags: `0` (zero-pad), `-` (left-align), `+` (sign), ` ` (space sign), `#` (alternate form).
- Dynamic precision: `.*` is supported. (Dynamic width `*` is not implemented).
- Unsupported conversions: `%p` and `%n`.
- NaN and Infinities: Zero padding falls back to spaces; sign of NaN is implementation-defined.
