#include <cstdio>

template <typename... Ts>
struct Count;

template <>
struct Count<> {
    static int value() { return 0; }
};

template <typename Head, typename... Tail>
struct Count<Head, Tail...> {
    static int value() {
        return 1 + Count<Tail...>::value();
    }
};

int main() {
    printf("%d\n", Count<>::value());
    printf("%d\n", Count<int>::value());
    printf("%d\n", Count<int, double, char>::value());
    return 0;
}
