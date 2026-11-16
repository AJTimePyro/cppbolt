#include <iostream>

template<int N>
struct IntConst {
    static const int value = N;
};

template<typename... Ts>
struct VariadicCounter {
    static const int count = IntConst<sizeof...(Ts)>::value;
    static const int count_plus_two = IntConst<sizeof...(Ts) + 2>::value;

    int get_count() const {
        return IntConst<sizeof...(Ts)>::value;
    }
};

template<typename... Ts>
void test_func() {
    std::cout << IntConst<sizeof...(Ts)>::value << std::endl;
}

int main() {
    VariadicCounter<> c0;
    VariadicCounter<int> c1;
    VariadicCounter<int, double, char> c3;

    std::cout << VariadicCounter<>::count << ' ' << VariadicCounter<>::count_plus_two << ' ' << c0.get_count() << std::endl;
    std::cout << VariadicCounter<int>::count << ' ' << VariadicCounter<int>::count_plus_two << ' ' << c1.get_count() << std::endl;
    std::cout << VariadicCounter<int, double, char>::count << ' ' << VariadicCounter<int, double, char>::count_plus_two << ' ' << c3.get_count() << std::endl;

    test_func<>();
    test_func<int, int>();
    return 0;
}
