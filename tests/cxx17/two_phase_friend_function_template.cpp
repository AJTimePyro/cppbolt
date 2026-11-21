#include <cstdio>

namespace N {
template <class T> struct S {
    template <class U> friend void probe(S, U) { std::printf("S probe\n"); }
};
}

int main() {
    N::S<int> a;
    N::S<double> b;
    probe(a, 1);
    probe(a, 2.0);
    probe(b, 1);
    probe(a, 1);
    probe(b, 2.0);
    return 0;
}
