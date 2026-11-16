#include <cstdio>
#include <tuple>

template <typename... Ts>
using MyTuple = std::tuple<Ts...>;

template <typename First, typename... Rest>
using PairTuple = std::tuple<First, std::tuple<Rest...>>;

int main() {
    MyTuple<int, double, char> t(1, 2.0, 'a');
    std::printf("%d %c\n", std::get<0>(t), std::get<2>(t));

    PairTuple<int, double, char> p(10, std::make_tuple(3.14, 'z'));
    std::printf("%d %c\n", std::get<0>(p), std::get<1>(std::get<1>(p)));
    return 0;
}
