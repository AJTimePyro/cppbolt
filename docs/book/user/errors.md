# Diagnostics & Compiler Errors

All diagnostic messages in `cppbolt` are represented by the unified `CompileError` struct. Error formatting follows GCC and Clang conventions:

```text
file.cpp:line:column: error: message
```

Diagnostics never expose Rust internal debug formatting (`Debug`) or panics.

---

## 1. Diagnostic Architecture

### Source Coordinate Fidelity
1. **Preprocessor Origins**: When files are included via `#include`, the preprocessor records line origin metadata (`src/preprocessor/lines.rs`). Errors originating from header files report the path and line number of the header itself rather than the combined translation unit.
2. **Token Spans**: The lexer attaches a precise `Span` (`FileId`, line, column, length) to every emitted token.
3. **Expression Precision**: Every AST expression node stores an `ExprPos` holding the exact line and column of its initial token. Diagnostics for invalid member accesses, dereferences, or conversions point directly at the offending expression.
4. **Statement Lowering Coordinates**: Code generation errors that cannot name a sub-expression report the source position of the parent statement.

The corpus test `tests/diagnostic_positions.rs` compiles every corpus file and enforces that no diagnostic reports a fabricated line or column.

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

The following catalog lists diagnostics emitted by the compiler, paired with real triggers from `tests/negative/`:

### Unsupported C++ Features (`ErrorKind::Unsupported`)

#### `multiple inheritance is not supported`
- **Cause**: A class declaration attempts to inherit from more than one base class.
- **Trigger** (`tests/negative/multiple_inheritance.cpp`):
  ```cpp
  struct A {};
  struct B {};
  struct C : public A, public B {}; // error: multiple inheritance is not supported
  ```

#### `virtual inheritance is not supported`
- **Cause**: A base class specifier includes the `virtual` keyword.
- **Trigger** (`tests/negative/virtual_inheritance.cpp`):
  ```cpp
  struct Base {};
  struct Derived : virtual public Base {}; // error: virtual inheritance is not supported
  ```

#### `range expression must be an array or class with begin() and end()`
- **Cause**: Attempting a range-based `for` loop on a scalar type or a class lacking `.begin()` and `.end()` methods.
- **Trigger** (`tests/negative/range_for_non_iterable.cpp`):
  ```cpp
  int main() {
      int total = 0;
      for (int x : 42) { // error: range expression must be an array or class with begin() and end(), found 'int'
          total += x;
      }
      return total;
  }
  ```

#### `Arrow member access on non-pointer`
- **Cause**: Using `->` on a struct or primitive variable that does not define an overloaded `operator->()`.
- **Trigger** (`tests/negative/operator_arrow_invalid.cpp`):
  ```cpp
  struct Dummy { int val; };
  int main() {
      Dummy d(1);
      return d->val; // error: Arrow member access on non-pointer
  }
  ```

#### `circular operator-> delegation on '<class>'`
- **Cause**: Recursive `operator->()` chaining encounters a cyclical delegation loop (e.g. `A -> B -> A` or `A -> A`).
- **Trigger** (`tests/negative/operator_arrow_cycle.cpp`):
  ```cpp
  struct B;
  struct A { B operator->(); };
  struct B { A operator->(); };
  int main() {
      A a;
      return a->val; // error: circular operator-> delegation on 'A'
  }
  ```

#### `operator-> must return a pointer to a struct or class`
- **Cause**: Overloaded `operator->()` returns a primitive or non-pointer type instead of a class pointer or wrapper object.
- **Trigger** (`tests/negative/operator_arrow_non_pointer.cpp`):
  ```cpp
  struct Bad {
      int operator->() { return 42; }
  };
  int main() {
      Bad b;
      return b->val; // error: operator-> must return a pointer to a struct or class
  }
  ```

#### `reference to '<name>' is ambiguous`
- **Cause**: An unqualified identifier is provided by two active `using namespace` directives or using declarations with equal precedence.
- **Trigger** (`tests/negative/ambiguous_using_directives.cpp`):
  ```cpp
  namespace A { int shared = 1; }
  namespace B { int shared = 2; }
  using namespace A;
  using namespace B;
  int main() { return shared; } // error: reference to 'shared' is ambiguous
  ```

#### `call of overloaded '<name>(...)' is ambiguous`
- **Cause**: Overload resolution finds multiple candidate functions with identical ranking scores.
- **Trigger** (`tests/negative/ambiguous_overload_call.cpp`):
  ```cpp
  void pick(float);
  void pick(long long);
  int main() {
      pick(1.0); // error: call of overloaded 'pick(double)' is ambiguous
  }
  ```

#### `lvalue required as left operand of assignment`
- **Cause**: Attempting to assign a value to an rvalue expression or temporary.
- **Trigger** (`tests/negative/assign_to_rvalue.cpp`):
  ```cpp
  int main() {
      1 = 2; // error: lvalue required as left operand of assignment
  }
  ```

#### `Cannot assign to variable '<name>' with const-qualified type`
- **Cause**: Mutating a variable declared with `const`.
- **Trigger** (`tests/negative/const_assign.cpp`):
  ```cpp
  int main() {
      const int c = 5;
      c = 6; // error: Cannot assign to variable 'c' with const-qualified type
  }
  ```

#### `no matching conversion for argument N of '<func>': cannot convert '<type1>' to '<type2>'`
- **Cause**: Passing an argument whose type cannot be converted to the parameter type through standard promotions or converting constructors.
- **Trigger** (`tests/negative/function_int_to_struct.cpp`):
  ```cpp
  struct Other { Other(const Other&); };
  void takes_struct(Other);
  int main() {
      takes_struct(1); // error: no matching conversion for argument 1 of 'takes_struct': cannot convert 'int' to 'Other'
  }
  ```

#### `no matching constructor for initialization of '<class>'`
- **Cause**: Creating an object where provided arguments do not match any constructor overload.
- **Trigger** (`tests/negative/constructor_too_many_arguments.cpp`):
  ```cpp
  struct T { T(int); };
  int main() {
      T x(1, 2); // error: no matching constructor for initialization of 'T'
  }
  ```

#### `invalid operands to binary expression: two pointers cannot be added`
- **Cause**: Applying the binary `+` operator between two pointer operands.
- **Trigger** (`tests/negative/pointer_plus_pointer.cpp`):
  ```cpp
  int main() {
      int a = 1, b = 2;
      return (&a + &b) != 0; // error: invalid operands to binary expression: two pointers cannot be added
  }
  ```

#### `'<member>' is a private member of '<class>'`
- **Cause**: Accessing a private struct or class member from outside its member methods.
- **Trigger** (`tests/negative/private_access.cpp`):
  ```cpp
  class Secret { int hidden; };
  int main() {
      Secret s;
      return s.hidden; // error: 'hidden' is a private member of 'Secret'
  }
  ```

#### `Variable type '<class>' is an abstract class`
- **Cause**: Instantiating an object of a class containing un-overridden pure virtual functions (`= 0`).
- **Trigger** (`tests/negative/pure_virtual_instantiation.cpp`):
  ```cpp
  struct Abstract { virtual void run() = 0; };
  int main() {
      Abstract a; // error: Variable type 'Abstract' is an abstract class
  }
  ```

#### `narrowing conversion from '<type1>' to '<type2>' inside braced-init-list`
- **Cause**: Converting between incompatible types inside a braced initializer list (e.g. floating-point to integer).
- **Trigger** (`tests/negative/narrowing_float_to_int.cpp`):
  ```cpp
  int main() {
      int x{3.14}; // error: narrowing conversion from 'double' to 'int' inside braced-init-list
      return x;
  }
  ```

#### `narrowing conversion of constant <val> from '<type1>' to '<type2>' inside braced-init-list`
- **Cause**: Converting a constant expression or literal to an integer type whose representable range cannot accommodate the value.
- **Trigger** (`tests/negative/narrowing_constant_overflow.cpp`):
  ```cpp
  int main() {
      char c{300}; // error: narrowing conversion of constant 300 from 'int' to 'char' inside braced-init-list
      return c;
  }
  ```

#### `chosen constructor for '<class>' is explicit in copy-list-initialization`
- **Cause**: An `explicit` constructor is matched during copy-list-initialization (`Type x = {...}`, `return {...}`, `fn({...})`).
- **Trigger** (`tests/negative/explicit_ctor_copy_list.cpp`):
  ```cpp
  struct Foo {
      explicit Foo(int) {}
  };
  int main() {
      Foo f = {42}; // error: chosen constructor for 'Foo' is explicit in copy-list-initialization
      return 0;
  }
  ```

#### `scalar initializer has multiple expressions`
- **Cause**: Initializing a scalar or pointer variable with a braced list containing more than one element.
- **Trigger** (`tests/negative/scalar_excess_elements.cpp`):
  ```cpp
  int main() {
      int x{1, 2}; // error: scalar initializer has multiple expressions
      return x;
  }
  ```

#### `too many initializers for aggregate '<class>'`
- **Cause**: Providing more initializer elements in a braced list than the target aggregate class has fields or subobjects.
- **Trigger** (`tests/negative/aggregate_excess_elements.cpp`):
  ```cpp
  struct Point { int x; int y; };
  int main() {
      Point p{1, 2, 3}; // error: too many initializers for aggregate 'Point'
      return 0;
  }
  ```

#### `excess elements in array initializer: expected at most <N>, have <M>`
- **Cause**: A dynamic array allocation `new T[N]{...}` specifies more initializer expressions than the declared array bound `N`.
- **Trigger** (`tests/negative/new_array_excess_elements.cpp`):
  ```cpp
  int main() {
      int* p = new int[2]{1, 2, 3}; // error: excess elements in array initializer: expected at most 2, have 3
      return 0;
  }
  ```

#### `cannot initialize variable '<name>' of type <target> with <source>`
- **Cause**: Initializing a typed variable with an incompatible expression, such as assigning a non-zero integer or `bool` to a pointer, discarding `const` qualifiers, violating multi-level pointer qualification rules, or assigning a string literal to a non-const `char*`.
- **Trigger** (`tests/negative/pointer_conversion_int_nonzero.cpp`):
  ```cpp
  int main() {
      int* p = 42; // error: cannot initialize variable 'p' of type int * with int
      return 0;
  }
  ```

### Move Semantics & Value Categories (`ErrorKind::Semantic`)

#### `cannot bind rvalue reference of type '<type>' to lvalue of type '<type>'`
- **Cause**: Attempting to bind an rvalue reference `T&&` directly to an lvalue, such as a named local variable or named rvalue reference parameter.
- **Trigger** (`tests/negative/rvalue_ref_bind_lvalue.cpp`):
  ```cpp
  int main() {
      int x = 42;
      int&& r = x; // error: cannot bind rvalue reference of type 'int &&' to lvalue of type 'int'
      return 0;
  }
  ```

#### `cannot initialize variable '<name>' of type <target> with <source>` (Move-Only Type)
- **Cause**: Copying an object whose class declares move operations, which suppresses implicit copy construction.
- **Trigger** (`tests/negative/unique_ptr_copy.cpp`):
  ```cpp
  #include <memory>
  int main() {
      std::unique_ptr<int> p1(new int(42));
      std::unique_ptr<int> p2 = p1; // error: cannot initialize variable 'p2' of type unique_ptr<int> with unique_ptr<int>
      return 0;
  }
  ```

#### `Type mismatch in assignment: cannot assign <source> to <target>`
- **Cause**: Assigning an lvalue of a move-only type to another object of the same type via copy assignment.
- **Trigger** (`tests/negative/unique_ptr_assign.cpp`):
  ```cpp
  #include <memory>
  int main() {
      std::unique_ptr<int> p1(new int(10));
      std::unique_ptr<int> p2(new int(20));
      p2 = p1; // error: Type mismatch in assignment: cannot assign unique_ptr<int> to unique_ptr<int>
      return 0;
  }
  ```

### Templates & Compile-Time Evaluation (`ErrorKind::Semantic`)

#### `redefinition of explicit template specialization '<spec>'`
- **Cause**: Declaring duplicate explicit (full) template specializations for identical argument types.
- **Trigger** (`tests/negative/template_spec_duplicate.cpp`):
  ```cpp
  template <typename T> struct DupSpec { int value() { return 1; } };
  template <> struct DupSpec<int> { int value() { return 2; } };
  template <> struct DupSpec<int> { int value() { return 3; } }; // error: redefinition of explicit template specialization 'DupSpec<int>'
  ```

#### `ambiguous partial specializations for '<instantiation>'`
- **Cause**: Two or more partial specializations match a template argument list with equal specificity scores.
- **Trigger** (`tests/negative/template_partial_spec_ambiguous.cpp`):
  ```cpp
  template <typename T, typename U> struct Ambiguous { static int run() { return 0; } };
  template <typename T> struct Ambiguous<T, int> { static int run() { return 1; } };
  template <typename U> struct Ambiguous<int, U> { static int run() { return 2; } };
  int main() {
      return Ambiguous<int, int>::run(); // error: ambiguous partial specializations for 'Ambiguous<int, int>'
  }
  ```

#### `type template parameter '<param>' cannot take non-type argument`
- **Cause**: Passing a constant value or non-type argument to a template parameter declared as `typename` or `class`.
- **Trigger** (`tests/negative/template_type_arg_mismatch.cpp`):
  ```cpp
  template <typename T> struct Box { T val; };
  int main() {
      Box<42> b; // error: type template parameter 'T' cannot take non-type argument
      return 0;
  }
  ```

#### `template argument for non-type parameter must be a constant expression`
- **Cause**: Supplying a string literal or incompatible expression to an integral non-type template parameter.
- **Trigger** (`tests/negative/template_nttp_type_mismatch.cpp`):
  ```cpp
  template <typename T, int N> struct Array { T data[N]; };
  int main() {
      Array<int, "hello"> a; // error: template argument for non-type parameter must be a constant expression
      return 0;
  }
  ```

#### `template parameter without default follows parameter with default`
- **Cause**: Declaring a template parameter without a default argument after a parameter declaring a default argument.
- **Trigger** (`tests/negative/template_default_arg_order.cpp`):
  ```cpp
  template <typename T = int, typename U> // error: template parameter without default follows parameter with default
  struct InvalidOrder {};
  ```

#### `integer overflow in constant expression division`
- **Cause**: Division by zero or integer overflow during compile-time constant expression folding.
- **Trigger** (`tests/negative/constexpr_division_overflow.cpp`):
  ```cpp
  template <int N> struct C {};
  C<(-2147483647 - 1) / -1> c; // error: integer overflow in constant expression division
  ```

#### `shift count out of range in constant expression`
- **Cause**: Bitwise shift in a compile-time constant expression where the shift count is negative or exceeds the integer bit width.
- **Trigger** (`tests/negative/constexpr_shift_overflow.cpp`):
  ```cpp
  template <int N> struct C {};
  C<1 << 33> c; // error: shift count out of range in constant expression
  ```

---

### Preprocessor & Lexical Errors (`ErrorKind::Lexical`)

#### `Invalid digit '<digit>' in binary literal`
- **Cause**: Non-binary digit in a binary integer literal (`0b...`).
- **Trigger** (`tests/negative/invalid_binary_digit.cpp`):
  ```cpp
  int main() {
      int x = 0b102; // error: Invalid digit '2' in binary literal
      return x;
  }
  ```

#### `Invalid digit separator`
- **Cause**: Consecutive digit separators or quotes at literal boundaries.
- **Trigger** (`tests/negative/invalid_digit_separator.cpp`):
  ```cpp
  int main() {
      int x = 1''000; // error: Invalid digit separator
      return x;
  }
  ```

#### `Invalid digit '<digit>' in octal literal`
- **Cause**: Non-octal digit (`8` or `9`) in a zero-prefixed octal literal.
- **Trigger** (`tests/negative/invalid_octal_digit.cpp`):
  ```cpp
  int main() {
      int x = 088; // error: Invalid digit '8' in octal literal
      return x;
  }
  ```

#### `include cycle detected: '<path>'`
- **Cause**: Cyclic `#include` inclusion chain detected by the preprocessor.
- **Trigger** (`tests/negative/include_cycle.cpp`):
  ```cpp
  #include "include_cycle_a.h" // a includes b, which includes a
  // error: include cycle detected: 'tests/negative/include_cycle_a.h'
  ```

#### `#endif without #if`
- **Cause**: A stray `#endif` encountered without an active conditional block.
- **Trigger** (`tests/negative/stray_endif.cpp`):
  ```cpp
  #endif // error: #endif without #if
  ```

#### `Unterminated #if/#ifdef/#ifndef block at EOF`
- **Cause**: Reaching end of file while a conditional compilation block remains unclosed.
- **Trigger** (`tests/negative/unterminated_if.cpp`):
  ```cpp
  #ifdef MISSING
  // error: Unterminated #if/#ifdef/#ifndef block at EOF
  ```

---

### Resource & Safety Diagnostics (`ErrorKind::Internal`)

#### `Nesting depth exceeds the supported limit`
- **Cause**: An input program exceeds calibrated recursion depth limits (e.g. >1,600 nested parentheses or >1,599 nested `if` blocks) to protect the WebAssembly shadow stack.
- **Trigger** (`tests/negative/nesting_depth_limit.cpp`):
  ```cpp
  // 1601 nested parentheses: (((...(((1)))...)))
  // error: Nesting depth exceeds the supported limit
  ```
