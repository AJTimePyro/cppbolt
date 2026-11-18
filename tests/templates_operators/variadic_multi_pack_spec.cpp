#include <stdio.h>

template <typename... Ts>
struct TypeList {};

template <typename ListA, typename ListB>
struct ZipCount;

template <typename... As, typename... Bs>
struct ZipCount<TypeList<As...>, TypeList<Bs...>> {
    static const int count = sizeof...(As) + sizeof...(Bs);
};

int main() {
    printf("ZipCount: %d\n", ZipCount<TypeList<int, double, char>, TypeList<short, float>>::count);
    return 0;
}
