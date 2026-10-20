# Language support

The supported v1 subset is intentionally small: integer and floating-point scalars, pointers, references,
arrays, enums, structs/classes, single and linear multiple-level inheritance, virtual functions, RAII, control
flow, overloads, default arguments, namespaces, selected operator overloads, `new`/`delete`, and the
preprocessor. The mini library covers `iostream`, `string`, `cstdio`, `cmath`, and `iomanip`.

Templates, `vector`, `map`, `set`, `algorithm`, lambdas, exceptions, RTTI, threads, SFINAE, variadic templates,
concepts, rvalue references, friends, and virtual inheritance are not implemented; a virtual base specifier is
rejected with `virtual inheritance is not supported`. Multiple inheritance is limited to data layout and
constructors, with non-static method lookup following the first base class. The tests in
`tests/templates_operators` and the container portion of `tests/mini_stl` document these boundaries.

Every supported behavior must have a positive corpus test. Unsupported syntax must be rejected by parser or
semantic diagnostics, not by searching source text.
