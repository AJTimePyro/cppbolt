#include <cstdio>

namespace N {
struct S {};
S first(S) { std::printf("first\n"); return S{}; }
void second(S) { std::printf("second\n"); }
}

// Nested calls and calls in nested expressions: both positions are classified
// on the definition AST and survive substitution.
template <class T> void nested(T t) { second(first(t)); }

// Two calls on one line at different columns: the dependent one sees the later
// overload, the non-dependent one keeps the definition-time overload.
namespace M {
struct A {};
void pick(const A&) { std::printf("def\n"); }
}
template <class T> void same_line(T t) {
    M::A a;
    pick(a); pick(t);
}
namespace M { void pick(A&) { std::printf("late\n"); } }

int main() {
    nested(N::S{});
    M::A a;
    same_line(a);
    return 0;
}
