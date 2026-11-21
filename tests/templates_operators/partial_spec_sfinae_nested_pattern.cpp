#include <cstdio>

template <typename T> struct Box {};
struct Yes { using type = int; };
struct No {};

template <typename T, typename U> struct H { static const int v = 0; };
template <typename T> struct H<T, Box<decltype(alignof(typename T::type))>> {
    static const int v = 1;
};

int main() {
    std::printf("%d %d\n", H<Yes, Box<size_t>>::v, H<No, Box<size_t>>::v);
    return 0;
}
