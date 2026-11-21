#include <cstdio>

struct Has { int value; };
struct No {};
struct S {
    template <typename T> auto get(T x) -> decltype(x.value) { return 1; }
    int get(No) { return 2; }
};

int main() {
    S s;
    Has h;
    No n;
    std::printf("%d %d\n", s.get(h), s.get(n));
    return 0;
}
