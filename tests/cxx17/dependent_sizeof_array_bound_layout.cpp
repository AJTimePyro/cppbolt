#include <cstdio>

struct Big { int a, b, c; };
struct Wide { double d; };
struct H { typedef Big type; };
struct HWide { typedef Wide type; };

template <class T>
struct S { char data[sizeof(typename T::type)]; };

template <class T>
struct R { char data[sizeof(T)]; };

template <class T>
struct Q { char data[alignof(typename T::type)]; };

template <class T>
struct C { char data[sizeof(typename T::type) + 1]; };

S<H> s;
R<Big> r;
Q<H> q;
Q<HWide> qw;
C<H> c;

int main() {
    printf("%zu %zu %zu %zu %zu\n", sizeof(s.data), sizeof(r.data),
           sizeof(q.data), sizeof(qw.data), sizeof(c.data));
    return 0;
}
