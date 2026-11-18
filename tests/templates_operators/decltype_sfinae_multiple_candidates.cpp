#include <cstdio>

struct A { int size() { return 0; } };
struct B { int value; };

// Two candidates share parameter types but differ in their trailing return
// types; both must stay registered and only the valid one is formed per call.
template <typename T> auto pick(T x) -> decltype(x.size()) { return 1; }
template <typename T> auto pick(T x) -> decltype(x.value) { return 2; }

int main() {
    A a;
    B b;
    std::printf("%d %d\n", pick(a), pick(b));
    return 0;
}
