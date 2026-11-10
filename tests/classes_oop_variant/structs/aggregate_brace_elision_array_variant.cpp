#include <cstdio>

struct Pair4 {
    int v[4];
};

struct Outer {
    Pair4 first;
    Pair4 second;
};

Pair4 g_p4{100, 200, 300, 400};
Outer g_outer{1, 2, 3, 4, 5, 6, 7, 8};

int main() {
    Pair4 l_part{55, 66};
    std::printf("l_part: %d %d %d %d\n", l_part.v[0], l_part.v[1], l_part.v[2], l_part.v[3]);

    std::printf("g_p4: %d %d %d %d\n", g_p4.v[0], g_p4.v[1], g_p4.v[2], g_p4.v[3]);

    std::printf("g_outer: %d %d %d %d | %d %d %d %d\n",
        g_outer.first.v[0], g_outer.first.v[1], g_outer.first.v[2], g_outer.first.v[3],
        g_outer.second.v[0], g_outer.second.v[1], g_outer.second.v[2], g_outer.second.v[3]);

    Outer l_outer{9, 10, 11, 12, 13, 14};
    std::printf("l_outer: %d %d %d %d | %d %d %d %d\n",
        l_outer.first.v[0], l_outer.first.v[1], l_outer.first.v[2], l_outer.first.v[3],
        l_outer.second.v[0], l_outer.second.v[1], l_outer.second.v[2], l_outer.second.v[3]);

    return 0;
}
