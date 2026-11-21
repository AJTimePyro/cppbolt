#include <cstdio>

namespace N { void f(int) { std::printf("N::f\n"); } }
using N::f;

template <class T> void caller(T) { f(1); }

int main() {
    caller(1);
    return 0;
}
