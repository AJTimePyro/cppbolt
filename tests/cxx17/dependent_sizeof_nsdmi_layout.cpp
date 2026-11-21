#include <cstdio>

struct SmallT { typedef char type; };
struct BigT { typedef long long type; };

template <class T>
struct S {
    int n = sizeof(typename T::type);
    static const int sc = sizeof(typename T::type);
    static const int al = alignof(typename T::type);
};

int main() {
    S<SmallT> small;
    S<BigT> big;
    printf("%d %d %d | %d %d %d\n", small.n, S<SmallT>::sc, S<SmallT>::al,
           big.n, S<BigT>::sc, S<BigT>::al);
    return 0;
}
