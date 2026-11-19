#include <cstdio>

// Eligible static const integral members of plain classes fold to constants
// and can be used as non-type template arguments.

struct C {
    static const int n = 7;
    static const int m = 3;
};

namespace NS {
struct D { static const int k = 11; };
}

struct Base { static const int h = 13; };
struct Derived : Base {};

constexpr int base = 5;
struct E { static const int v = base + 1; };

template<int N> struct Box { static constexpr int value = N; };
template<class T, int N> struct Wrap { static constexpr int value = N; };
template<int N> struct Outer { static constexpr int value = Box<N>::value * 2; };

int main() {
    std::printf("Box<C::n>     = %d\n", Box<C::n>::value);
    std::printf("Box<C::m>     = %d\n", Box<C::m>::value);
    std::printf("Box<NS::D::k> = %d\n", Box<NS::D::k>::value);
    std::printf("Box<::C::n>   = %d\n", Box<::C::n>::value);
    std::printf("Box<Derived::h> = %d\n", Box<Derived::h>::value);
    std::printf("Box<C::n + 1> = %d\n", Box<C::n + 1>::value);
    std::printf("Wrap<C, C::m> = %d\n", Wrap<C, C::m>::value);
    std::printf("Outer<C::n>   = %d\n", Outer<C::n>::value);
    std::printf("Box<E::v>     = %d\n", Box<E::v>::value);
    return 0;
}
