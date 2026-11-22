#include <cstdio>
#include <array>

int main() {
    std::array<int, 3> x1 = {100, 200, 300};
    std::array<int, 3> x2 = {{100, 200, 300}};
    std::array<int, 3> x3 = {100, 250, 300};

    std::printf("eq: %d %d\n", x1 == x2, x1 == x3);
    std::printf("ne: %d %d\n", x1 != x2, x1 != x3);
    std::printf("lt: %d %d\n", x1 < x3, x3 < x1);

    std::printf("rev:");
    for (auto rit = x1.rbegin(); rit != x1.rend(); ++rit) {
        std::printf(" %d", *rit);
    }
    std::printf("\n");

    std::printf("front_back: %d %d\n", x1.front(), x1.back());
    std::printf("at: %d\n", x1.at(1));
    std::printf("get: %d %d %d\n", std::get<0>(x1), std::get<1>(x1), std::get<2>(x1));

    x2.fill(42);
    std::printf("fill: %d %d %d\n", x2[0], x2[1], x2[2]);
    x1.swap(x2);
    std::printf("swap: %d %d\n", x1[0], x2[0]);

    std::array<char, 0> z;
    std::printf("zero: %zu %d\n", z.size(), z.empty());

    return 0;
}
