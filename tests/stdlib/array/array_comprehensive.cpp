#include <cstdio>
#include <array>

int main() {
    std::array<int, 4> a1 = {10, 20, 30, 40};
    std::array<int, 4> a2 = {{10, 20, 30, 40}};
    std::array<int, 4> a3 = {10, 20, 35, 40};

    // Comparisons
    std::printf("eq: %d %d\n", a1 == a2, a1 == a3);
    std::printf("ne: %d %d\n", a1 != a2, a1 != a3);
    std::printf("lt: %d %d\n", a1 < a3, a3 < a1);
    std::printf("le: %d %d\n", a1 <= a2, a1 <= a3);
    std::printf("gt: %d %d\n", a3 > a1, a1 > a3);
    std::printf("ge: %d %d\n", a1 >= a2, a3 >= a1);

    // Forward iterators
    std::printf("elems:");
    for (auto it = a1.begin(); it != a1.end(); ++it) {
        std::printf(" %d", *it);
    }
    std::printf("\n");

    // Reverse iterators
    std::printf("reverse:");
    for (auto rit = a1.rbegin(); rit != a1.rend(); ++rit) {
        std::printf(" %d", *rit);
    }
    std::printf("\n");

    // Element access
    std::printf("front_back: %d %d\n", a1.front(), a1.back());
    std::printf("at: %d %d\n", a1.at(1), a1.at(2));
    std::printf("data: %d\n", *a1.data());

    // Tuple get
    std::printf("get: %d %d %d %d\n", std::get<0>(a1), std::get<1>(a1), std::get<2>(a1), std::get<3>(a1));

    // Modifiers
    a2.fill(7);
    std::printf("fill: %d %d %d %d\n", a2[0], a2[1], a2[2], a2[3]);
    a1.swap(a2);
    std::printf("swap: %d %d\n", a1[0], a2[0]);

    // Zero-sized array
    std::array<int, 0> z;
    std::printf("zero: %zu %d %d\n", z.size(), z.empty(), z.data() == nullptr);

    return 0;
}
