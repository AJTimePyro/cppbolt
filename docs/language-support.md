# Language support

The supported v1 subset is intentionally small: integer and floating-point scalars, pointers, references,
arrays, enums, structs/classes, single and linear multiple-level inheritance, virtual functions, RAII, control
flow, overloads, default arguments, namespaces, selected operator overloads, `new`/`delete`, and the
preprocessor. The mini library covers `iostream`, `string`, `cstdio`, `cmath`, and `iomanip`.

Templates and the standard containers and algorithms (`vector`, `map`, `set`, `algorithm`) are not
implemented; the partial container headers cover only the subset used by the mini-STL fixtures. Lambdas,
exceptions, RTTI, threads, SFINAE, variadic templates, concepts, rvalue references, friends, and virtual
inheritance are also not implemented; a virtual base specifier is rejected with `virtual inheritance is not
supported`. Multiple inheritance is rejected at the second base specifier with `multiple inheritance is not
supported`; single inheritance and linear inheritance chains remain supported. The tests in
`tests/templates_operators` and the container portion of `tests/mini_stl` document these boundaries.

Every supported behavior must have a positive corpus test. Unsupported syntax must be rejected by parser or
semantic diagnostics, not by searching source text.

## Copy elision

A prvalue of a class initializes its destination directly, as C++17 guarantees, so the copy constructor does not
run for `T x(T(1))`, `T x = T(1)`, `T x(f())`, or `T x = f()` when `f` returns a prvalue. An lvalue initializes
its destination with the copy constructor, so `T x(y)` runs it. The copy of a named local in a `return` statement
is not elided: the standard requires the copy constructor to run, and the compiler runs it.

## Argument conversions

The arguments of a constructor, a function, and a method must match a declared parameter. The compiler accepts
the implicit conversions of the subset: an arithmetic conversion, a derived class to its base class, an array to
a pointer, `nullptr` to a pointer, an enum to an integer, a pointer to `void *`, and an added `const`
qualifier. An argument that matches no parameter is rejected with `no matching constructor for initialization
of 'T'` or `no matching conversion for argument N of 'f'`.

One simplification remains: an integer is accepted where a pointer is expected, because the compiler does not
track the constant-expression rule of a null pointer constant.

## Ambiguity

A name that two using-directives provide is ambiguous, and a call whose arguments convert equally well to two
overloads is ambiguous. The overload ranking is the one the subset supports: an exact match beats a reference
binding, which beats a promotion, which beats a standard conversion, which beats a converting constructor. Two
overloads with the same rank make the call ambiguous.
