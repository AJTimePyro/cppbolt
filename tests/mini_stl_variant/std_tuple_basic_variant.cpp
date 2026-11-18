#include <cstdio>
#include <tuple>

int main() {
    std::tuple<int, int, int, int> t4(1, 2, 3, 4);
    printf("%d %d %d %d\n", std::get<0>(t4), std::get<1>(t4), std::get<2>(t4), std::get<3>(t4));
    return 0;
}
