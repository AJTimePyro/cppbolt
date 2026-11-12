#include <cstdio>

struct Pair4 {
    int v[4];
};

struct Outer {
    Pair4 first;
    Pair4 second;
};

struct VBase {
    int alpha;
    int beta;
};

struct VDerived : VBase {
    int gamma;
    int delta;
};

struct Coords {
    int u, v, w;
};

Pair4 g_p4{100, 200, 300, 400};
Outer g_outer{1, 2, 3, 4, 5, 6, 7, 8};
VDerived g_vder = { {500, 600}, 700, 800 };
Coords g_coords[2] = {1, 2, 3, 4, 5, 6};
int g_matrix[3][2] = {10, 20, 30, 40, 50, 60};

int main() {
    Pair4 l_part{55, 66};
    std::printf("l_part: %d %d %d %d\n", l_part.v[0], l_part.v[1], l_part.v[2], l_part.v[3]);

    std::printf("g_p4: %d %d %d %d\n", g_p4.v[0], g_p4.v[1], g_p4.v[2], g_p4.v[3]);

    std::printf("g_outer: %d %d %d %d | %d %d %d %d\n",
        g_outer.first.v[0], g_outer.first.v[1], g_outer.first.v[2], g_outer.first.v[3],
        g_outer.second.v[0], g_outer.second.v[1], g_outer.second.v[2], g_outer.second.v[3]);

    std::printf("g_vder: %d %d %d %d\n", g_vder.alpha, g_vder.beta, g_vder.gamma, g_vder.delta);
    std::printf("g_coords: %d %d %d | %d %d %d\n",
        g_coords[0].u, g_coords[0].v, g_coords[0].w,
        g_coords[1].u, g_coords[1].v, g_coords[1].w);
    std::printf("g_matrix: %d %d %d %d %d %d\n",
        g_matrix[0][0], g_matrix[0][1], g_matrix[1][0], g_matrix[1][1], g_matrix[2][0], g_matrix[2][1]);

    Outer l_outer{9, 10, 11, 12, 13, 14};
    std::printf("l_outer: %d %d %d %d | %d %d %d %d\n",
        l_outer.first.v[0], l_outer.first.v[1], l_outer.first.v[2], l_outer.first.v[3],
        l_outer.second.v[0], l_outer.second.v[1], l_outer.second.v[2], l_outer.second.v[3]);

    VDerived l_vder = { {150, 250}, 350, 450 };
    std::printf("l_vder: %d %d %d %d\n", l_vder.alpha, l_vder.beta, l_vder.gamma, l_vder.delta);

    Coords l_coords[2] = {7, 8, 9, 10, 11, 12};
    std::printf("l_coords: %d %d %d | %d %d %d\n",
        l_coords[0].u, l_coords[0].v, l_coords[0].w,
        l_coords[1].u, l_coords[1].v, l_coords[1].w);

    int l_matrix[3][2] = {11, 22, 33, 44, 55, 66};
    std::printf("l_matrix: %d %d %d %d %d %d\n",
        l_matrix[0][0], l_matrix[0][1], l_matrix[1][0], l_matrix[1][1], l_matrix[2][0], l_matrix[2][1]);

    return 0;
}
