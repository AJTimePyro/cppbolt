#include <cstdio>

struct HasInt { typedef int type; };
struct HasChar { typedef char type; };
struct Base { typedef int type; };
struct Derived : Base {};
template <class T> struct TBase { typedef T type; };
template <class T> struct TDerived : TBase<T> {};

template <class T>
struct Holder {
    typedef typename T::type value_type;
    char data[sizeof(typename T::type)];
    char grid[2][sizeof(typename T::type)];
    char grid2[sizeof(typename T::type)][2];
};

struct Concrete { char data[sizeof(HasInt::type)]; };

template <class T>
struct Simple { char data[sizeof(T)]; };

int main() {
    Holder<HasInt> a;
    Holder<HasChar> b;
    Holder<Derived> c;
    Holder<TDerived<int> > d;
    Concrete e;
    Simple<int> f;
    printf("%d %d %d %d | %d %d\n", (int)sizeof(a.data), (int)sizeof(b.data),
           (int)sizeof(c.data), (int)sizeof(d.data), (int)sizeof(e.data),
           (int)sizeof(f.data));
    printf("%d %d %d %d\n", (int)sizeof(a.grid), (int)sizeof(a.grid[0]),
           (int)sizeof(a.grid2), (int)sizeof(a.grid2[0]));
    return 0;
}
