#include <cstdio>
#include <cstddef>

template <typename... Ts>
struct AlignedHolder {
    alignas(Ts...) char buffer[64];
};

template <typename... Ts>
struct AlignedStruct {
    alignas(Ts...) int x;
};

template <typename... Ts>
struct alignas(Ts...) AlignedClass {
    char data;
};

int main() {
    std::printf("%zu\n", alignof(AlignedHolder<char, int, double>));
    std::printf("%zu\n", alignof(AlignedHolder<char, short>));
    std::printf("%zu\n", alignof(AlignedHolder<>));
    std::printf("%zu\n", alignof(AlignedStruct<char, double>));
    std::printf("%zu\n", alignof(AlignedClass<int, double>));
    return 0;
}
