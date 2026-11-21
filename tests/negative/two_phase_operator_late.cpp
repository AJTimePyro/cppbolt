#include <cstdio>
namespace N { struct A {}; }
template <class T> void f(T) {
    N::A a;
    a + a;
}
namespace N { int operator+(A, A) { std::printf("late\n"); return 0; } }
int main() { f(1); }
