#include <cstdio>
namespace N { struct A {}; }
template <class T> void f(T t) {
    N::A a;
    hidden(a); hidden(t);
}
namespace N { void hidden(A) { std::printf("late\n"); } }
int main() { N::A a; f(a); }
