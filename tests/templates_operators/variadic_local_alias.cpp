#include <iostream>

template<int N>
struct IntConst {
    static const int value = N;
};

template<typename... Ts>
struct Target {
    static const int count = IntConst<sizeof...(Ts)>::value;
    int get() const {
        return IntConst<sizeof...(Ts)>::value;
    }
};

template<typename... Ts>
int run_test() {
    using Res = Target<Ts...>;
    Res r;
    return Res::count + r.get();
}

int main() {
    std::cout << run_test<>() << std::endl;
    std::cout << run_test<int>() << std::endl;
    std::cout << run_test<int, double, char>() << std::endl;
    return 0;
}
