#include <cstdio>
#include <tuple>

int main() {
    auto t = std::make_tuple(10, 20, 30, 40, 50, 60, 70, 80);
    std::get<0>(t) = 11;
    std::get<7>(t) = 88;
    printf("%d %d %d %d %d %d %d %d\n",
           std::get<0>(t), std::get<1>(t), std::get<2>(t), std::get<3>(t),
           std::get<4>(t), std::get<5>(t), std::get<6>(t), std::get<7>(t));
    return 0;
}
