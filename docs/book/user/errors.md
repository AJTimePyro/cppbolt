# Diagnostics & Compiler Errors

All diagnostic messages in `cppwasm` are represented by the unified [`CompileError`](https://github.com/cppwasm/cpp_web/blob/main/src/diagnostics/mod.rs) struct. Error formatting follows GCC and Clang conventions:

```text
file.cpp:line:column: error: message
```

Diagnostics never expose Rust internal debug formatting (`Debug`) or panics.

---

## 1. Diagnostic Architecture

### Source Coordinate Fidelity
1. **Preprocessor Origins**: When files are included via `#include`, the preprocessor records line origin metadata ([`src/preprocessor/lines.rs`](https://github.com/cppwasm/cpp_web/blob/main/src/preprocessor/lines.rs)). Errors originating from header files report the path and line number of the header itself rather than the combined translation unit.
2. **Token Spans**: The lexer attaches a precise [`Span`](https://github.com/cppwasm/cpp_web/blob/main/src/source.rs) (`FileId`, line, column, length) to every emitted token.
3. **Expression Precision**: Every AST expression node stores an [`ExprPos`](https://github.com/cppwasm/cpp_web/blob/main/src/ast/expr.rs) holding the exact line and column of its initial token. Diagnostics for invalid member accesses, dereferences, or conversions point directly at the offending expression.
4. **Statement Lowering Coordinates**: Code generation errors that cannot name a sub-expression report the source position of the parent statement.

The corpus test [`tests/diagnostic_positions.rs`](https://github.com/cppwasm/cpp_web/blob/main/tests/diagnostic_positions.rs) compiles every corpus file and enforces that no diagnostic reports a fabricated line or column.

---

## 2. Diagnostic Categories (`ErrorKind`)

The compiler categorizes diagnostics into five distinct kinds:

```rust
pub enum ErrorKind {
    Lexical,     // Tokenization and preprocessor syntax errors
    Syntax,      // Parser grammar failures
    Semantic,    // Type checking, symbol resolution, and layout failures
    Unsupported, // Features outside the v1 C++ subset
    Internal,    // Internal compiler invariants or resource limits
}
```

---

## 3. Catalog of Common Diagnostics & Trigger Examples

The following catalog lists diagnostics emitted by the compiler, paired with real triggers from [`tests/negative/`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/):

### Unsupported C++ Features (`ErrorKind::Unsupported`)

#### `multiple inheritance is not supported`
- **Cause**: A class declaration attempts to inherit from more than one base class.
- **Trigger** ([`tests/negative/multiple_inheritance.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/multiple_inheritance.cpp)):
  ```cpp
  struct A {};
  struct B {};
  struct C : public A, public B {}; // error: multiple inheritance is not supported
  ```

#### `virtual inheritance is not supported`
- **Cause**: A base class specifier includes the `virtual` keyword.
- **Trigger** ([`tests/negative/virtual_inheritance.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/virtual_inheritance.cpp)):
  ```cpp
  struct Base {};
  struct Derived : virtual public Base {}; // error: virtual inheritance is not supported
  ```

#### `range-based for is not supported`
- **Cause**: Using C++11 range-for syntax (`for (elem : collection)`).
- **Trigger** ([`tests/negative/range_for_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/range_for_unsupported.cpp)):
  ```cpp
  int main() {
      int arr[3] = {1, 2, 3};
      for (int x : arr) {} // error: range-based for is not supported
  }
  ```

#### `operator() is not supported`
- **Cause**: Declaring a call operator overload on a class.
- **Trigger** ([`tests/negative/operator_call_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/operator_call_unsupported.cpp)):
  ```cpp
  struct F {
      int operator()(int x) { return x + 1; } // error: operator() is not supported
  };
  ```

#### `operator-> is not supported`
- **Cause**: Declaring an overloaded arrow member-access operator on a class.
- **Trigger** ([`tests/negative/operator_arrow_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/operator_arrow_unsupported.cpp)):
  ```cpp
  struct P {
      int value;
      int* operator->() { return &value; } // error: operator-> is not supported
  };
  ```

#### `operator* is not supported`
- **Cause**: Declaring an overloaded unary dereference operator on a class.
- **Trigger** ([`tests/negative/operator_deref_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/operator_deref_unsupported.cpp)):
  ```cpp
  struct P {
      int value;
      int operator*() const { return value; } // error: operator* is not supported
  };
  ```

#### `postfix ++/-- is not supported`
- **Cause**: Declaring an overloaded postfix increment or decrement operator (`operator++(int)`).
- **Trigger** ([`tests/negative/postfix_increment_unsupported.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/postfix_increment_unsupported.cpp)):
  ```cpp
  struct Counter {
      Counter operator++(int); // error: postfix ++/-- is not supported
  };
  ```

#### `namespace aliases are not supported`
- **Cause**: Attempting to alias a namespace via `namespace Alias = Original;`.
- **Trigger** ([`tests/negative/namespace_alias.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/namespace_alias.cpp)):
  ```cpp
  namespace N { int x; }
  namespace Alias = N; // error: namespace aliases are not supported
  ```

---

### Semantic Errors (`ErrorKind::Semantic`)

#### `reference to '<name>' is ambiguous`
- **Cause**: An unqualified identifier is provided by two active `using namespace` directives or using declarations with equal precedence.
- **Trigger** ([`tests/negative/ambiguous_using_directives.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/ambiguous_using_directives.cpp)):
  ```cpp
  namespace A { int shared = 1; }
  namespace B { int shared = 2; }
  using namespace A;
  using namespace B;
  int main() { return shared; } // error: reference to 'shared' is ambiguous
  ```

#### `call of overloaded '<name>(...)' is ambiguous`
- **Cause**: Overload resolution finds multiple candidate functions with identical ranking scores.
- **Trigger** ([`tests/negative/ambiguous_overload_call.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/ambiguous_overload_call.cpp)):
  ```cpp
  void pick(float);
  void pick(long long);
  int main() {
      pick(1.0); // error: call of overloaded 'pick(double)' is ambiguous
  }
  ```

#### `lvalue required as left operand of assignment`
- **Cause**: Attempting to assign a value to an rvalue expression or temporary.
- **Trigger** ([`tests/negative/assign_to_rvalue.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/assign_to_rvalue.cpp)):
  ```cpp
  int main() {
      1 = 2; // error: lvalue required as left operand of assignment
  }
  ```

#### `Cannot assign to variable '<name>' with const-qualified type`
- **Cause**: Mutating a variable declared with `const`.
- **Trigger** ([`tests/negative/const_assign.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/const_assign.cpp)):
  ```cpp
  int main() {
      const int c = 5;
      c = 6; // error: Cannot assign to variable 'c' with const-qualified type
  }
  ```

#### `no matching conversion for argument N of '<func>': cannot convert '<type1>' to '<type2>'`
- **Cause**: Passing an argument whose type cannot be converted to the parameter type through standard promotions or converting constructors.
- **Trigger** ([`tests/negative/function_int_to_struct.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/function_int_to_struct.cpp)):
  ```cpp
  struct Other { Other(const Other&); };
  void takes_struct(Other);
  int main() {
      takes_struct(1); // error: no matching conversion for argument 1 of 'takes_struct': cannot convert 'int' to 'Other'
  }
  ```

#### `no matching constructor for initialization of '<class>'`
- **Cause**: Creating an object where provided arguments do not match any constructor overload.
- **Trigger** ([`tests/negative/constructor_too_many_arguments.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/constructor_too_many_arguments.cpp)):
  ```cpp
  struct T { T(int); };
  int main() {
      T x(1, 2); // error: no matching constructor for initialization of 'T'
  }
  ```

#### `invalid operands to binary expression: two pointers cannot be added`
- **Cause**: Applying the binary `+` operator between two pointer operands.
- **Trigger** ([`tests/negative/pointer_plus_pointer.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/pointer_plus_pointer.cpp)):
  ```cpp
  int main() {
      int a = 1, b = 2;
      return (&a + &b) != 0; // error: invalid operands to binary expression: two pointers cannot be added
  }
  ```

#### `'<member>' is a private member of '<class>'`
- **Cause**: Accessing a private struct or class member from outside its member methods.
- **Trigger** ([`tests/negative/private_access.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/private_access.cpp)):
  ```cpp
  class Secret { int hidden; };
  int main() {
      Secret s;
      return s.hidden; // error: 'hidden' is a private member of 'Secret'
  }
  ```

#### `Variable type '<class>' is an abstract class`
- **Cause**: Instantiating an object of a class containing un-overridden pure virtual functions (`= 0`).
- **Trigger** ([`tests/negative/pure_virtual_instantiation.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/pure_virtual_instantiation.cpp)):
  ```cpp
  struct Abstract { virtual void run() = 0; };
  int main() {
      Abstract a; // error: Variable type 'Abstract' is an abstract class
  }
  ```

---

### Preprocessor & Lexical Errors (`ErrorKind::Lexical`)

#### `include cycle detected: '<path>'`
- **Cause**: Cyclic `#include` inclusion chain detected by the preprocessor.
- **Trigger** ([`tests/negative/include_cycle.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/include_cycle.cpp)):
  ```cpp
  #include "include_cycle_a.h" // a includes b, which includes a
  // error: include cycle detected: 'tests/negative/include_cycle_a.h'
  ```

#### `#endif without #if`
- **Cause**: A stray `#endif` encountered without an active conditional block.
- **Trigger** ([`tests/negative/stray_endif.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/stray_endif.cpp)):
  ```cpp
  #endif // error: #endif without #if
  ```

#### `Unterminated #if/#ifdef/#ifndef block at EOF`
- **Cause**: Reaching end of file while a conditional compilation block remains unclosed.
- **Trigger** ([`tests/negative/unterminated_if.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/unterminated_if.cpp)):
  ```cpp
  #ifdef MISSING
  // error: Unterminated #if/#ifdef/#ifndef block at EOF
  ```

---

### Resource & Safety Diagnostics (`ErrorKind::Internal`)

#### `Nesting depth exceeds the supported limit`
- **Cause**: An input program exceeds calibrated recursion depth limits (e.g. >1,600 nested parentheses or >1,599 nested `if` blocks) to protect the WebAssembly shadow stack.
- **Trigger** ([`tests/negative/nesting_depth_limit.cpp`](https://github.com/cppwasm/cpp_web/blob/main/tests/negative/nesting_depth_limit.cpp)):
  ```cpp
  // 1601 nested parentheses: (((...(((1)))...)))
  // error: Nesting depth exceeds the supported limit
  ```
