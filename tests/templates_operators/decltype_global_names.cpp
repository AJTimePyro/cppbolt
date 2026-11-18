#include <cstdio>

// A global must be visible while field and parameter types are resolved.
int g = 3;
struct S { decltype(g) v; };
int f(decltype(g) p) { return p; }

int main() {
    S s;
    s.v = 4;
    std::printf("%d %d\n", s.v, f(5));
    return 0;
}
