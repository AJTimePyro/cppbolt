#include <cstdio>

void g(long) { std::printf("g(long)\n"); }

template <typename T> void call(T x) { g(x); }

// A better overload introduced after the template definition must not enter
// ordinary lookup for the call inside it.
void g(int) { std::printf("g(int)\n"); }

int main() {
    call(42L);
    call(42);
    g(42);
    return 0;
}
