#include <cstdio>

struct S {
    // The first candidate is discarded when T has no nested type; the second
    // candidate is selected for both calls without inheriting a stale method
    // registration from the rejected one.
    template <typename T> int f(T x, typename T::type* p) { return 1; }
    template <typename T> int f(T x, T* p) { return 2; }
};

int main() {
    S s;
    int a = 0;
    double b = 0;
    std::printf("%d %d\n", s.f(1, &a), s.f(2.0, &b));
    return 0;
}
