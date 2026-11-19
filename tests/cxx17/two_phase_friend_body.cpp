#include <cstdio>

void helper(int a, int b = 7) { std::printf("%d %d\n", a, b); }

namespace N {
template <class T> struct S {
    friend void f(S) { helper(1); }
    void call() { S s; f(s); }
};
}

int main() {
    N::S<int> s;
    s.call();
    f(N::S<int>{});
    return 0;
}
