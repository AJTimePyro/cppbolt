#include <cstdio>

template <typename... Ts>
struct Outer {
    template <typename... Us>
    struct Inner {
        static int count() {
            return sizeof...(Ts) * 10 + sizeof...(Us);
        }
    };
};

int main() {
    std::printf("%d\n", Outer<int, double>::Inner<char, float, void*>::count());
    std::printf("%d\n", Outer<int>::Inner<short, long>::count());
    std::printf("%d\n", Outer<>::Inner<>::count());
    return 0;
}
