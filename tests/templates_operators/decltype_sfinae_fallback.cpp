#include <cstdio>

struct Has { int value; };
struct No {};

template <typename T> auto f(T x) -> decltype(x.value) { return 1; }
int f(No) { return 2; }

int main() {
    Has h;
    No n;
    std::printf("%d %d\n", f(h), f(n));
    return 0;
}
