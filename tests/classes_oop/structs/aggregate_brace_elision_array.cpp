#include <cstdio>

struct Array3 {
    int elems[3];
};

struct Nested {
    int id;
    Array3 arr;
    int extra;
};

struct Array2D {
    int grid[2][3];
};

Array3 g_full{1, 2, 3};
Array3 g_partial{42};
Nested g_nested{100, 200, 300, 400, 500};

int main() {
    Array3 l_full{10, 20, 30};
    std::printf("l_full: %d %d %d\n", l_full.elems[0], l_full.elems[1], l_full.elems[2]);

    Array3 l_partial{99};
    std::printf("l_partial: %d %d %d\n", l_partial.elems[0], l_partial.elems[1], l_partial.elems[2]);

    std::printf("g_full: %d %d %d\n", g_full.elems[0], g_full.elems[1], g_full.elems[2]);
    std::printf("g_partial: %d %d %d\n", g_partial.elems[0], g_partial.elems[1], g_partial.elems[2]);
    std::printf("g_nested: %d %d %d %d %d\n", g_nested.id, g_nested.arr.elems[0], g_nested.arr.elems[1], g_nested.arr.elems[2], g_nested.extra);

    Nested l_nested{1, 2, 3, 4, 5};
    std::printf("l_nested: %d %d %d %d %d\n", l_nested.id, l_nested.arr.elems[0], l_nested.arr.elems[1], l_nested.arr.elems[2], l_nested.extra);

    Nested l_nested_part{7, 8};
    std::printf("l_nested_part: %d %d %d %d %d\n", l_nested_part.id, l_nested_part.arr.elems[0], l_nested_part.arr.elems[1], l_nested_part.arr.elems[2], l_nested_part.extra);

    Array2D l_2d{11, 22, 33, 44, 55, 66};
    std::printf("l_2d: %d %d %d %d %d %d\n", l_2d.grid[0][0], l_2d.grid[0][1], l_2d.grid[0][2], l_2d.grid[1][0], l_2d.grid[1][1], l_2d.grid[1][2]);

    return 0;
}
