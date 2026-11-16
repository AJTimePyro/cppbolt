#include <cstdio>

struct alignas(8) BigStruct {
    double d;
};

template<typename... Ts>
struct AlignedTypes {
    alignas(Ts...) char data;
};

template<typename... Ts>
struct AlignedExpr {
    alignas(alignof(Ts)...) char data;
};

template<typename... Ts>
struct AlignedPtr {
    alignas(alignof(Ts*)...) char data;
};

template<int... Ns>
struct AlignedNttp {
    alignas(Ns...) char data;
};

int main() {
    printf("%d\n", (int)alignof(AlignedTypes<char, short, int>));
    printf("%d\n", (int)alignof(AlignedTypes<char, double>));
    printf("%d\n", (int)alignof(AlignedTypes<char, BigStruct>));
    printf("%d\n", (int)alignof(AlignedExpr<char, short, int>));
    printf("%d\n", (int)alignof(AlignedExpr<char, double>));
    printf("%d\n", (int)alignof(AlignedPtr<char, double>));
    printf("%d\n", (int)alignof(AlignedNttp<1, 4, 2>));
    printf("%d\n", (int)alignof(AlignedNttp<2, 8>));
    return 0;
}
