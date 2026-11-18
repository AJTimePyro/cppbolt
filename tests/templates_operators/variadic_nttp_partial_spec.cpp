#include <stdio.h>

template <int... Values>
struct Sum;

template <>
struct Sum<> {
    static const int value = 0;
};

template <int First, int... Rest>
struct Sum<First, Rest...> {
    static const int value = First + Sum<Rest...>::value;
};

int main() {
    printf("Sum: %d\n", Sum<1, 2, 3, 4, 5>::value);
    return 0;
}
