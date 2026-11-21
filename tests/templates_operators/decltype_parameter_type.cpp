#include <cstdio>

// A decltype in a parameter type must not re-enter resolution with the same
// parameter list (that used to overflow the stack).
int g = 3;
template <typename T> int f(decltype(g) p) { return p; }

int main() {
    std::printf("%d\n", f<int>(7));
    return 0;
}
