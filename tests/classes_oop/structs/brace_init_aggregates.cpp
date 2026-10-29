#include <cstdio>

struct BaseAgg {
    int bx;
    int by;
};

struct DerivedAgg : BaseAgg {
    int dz;
};

struct Point {
    int x;
    int y;
    int z;
};

int main() {
    DerivedAgg da1{{1, 2}, 3};
    DerivedAgg da2{};
    std::printf("da1: %d, %d, %d\n", da1.bx, da1.by, da1.dz);
    std::printf("da2: %d, %d, %d\n", da2.bx, da2.by, da2.dz);

    Point p1{10};
    Point p2{};
    std::printf("p1: %d, %d, %d\n", p1.x, p1.y, p1.z);
    std::printf("p2: %d, %d, %d\n", p2.x, p2.y, p2.z);

    Point arr[3]{{1, 2, 3}, {4, 5, 6}};
    for (int i = 0; i < 3; ++i) {
        std::printf("arr[%d]: %d, %d, %d\n", i, arr[i].x, arr[i].y, arr[i].z);
    }

    return 0;
}
