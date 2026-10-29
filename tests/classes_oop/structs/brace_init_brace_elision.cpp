#include <cstdio>

struct BaseAgg {
    int bx;
    int by;
};

struct DerivedAgg : BaseAgg {
    int dz;
};

struct Sub {
    int a;
    int b;
};

struct Container {
    Sub s;
    int c;
};

int main() {
    DerivedAgg da_nested{{1, 2}, 3};
    DerivedAgg da_flat{10, 20, 30};
    std::printf("nested: %d %d %d\n", da_nested.bx, da_nested.by, da_nested.dz);
    std::printf("flat: %d %d %d\n", da_flat.bx, da_flat.by, da_flat.dz);

    Container c_nested{{100, 200}, 300};
    std::printf("c_nested: %d %d %d\n", c_nested.s.a, c_nested.s.b, c_nested.c);

    int m[2][2] = {{1, 2}, {3, 4}};
    std::printf("m: %d %d %d %d\n", m[0][0], m[0][1], m[1][0], m[1][1]);

    return 0;
}
