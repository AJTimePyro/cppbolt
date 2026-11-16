#include <cstdio>

template <typename... Inner>
int sum_inner(Inner... inners) {
    return (... + inners);
}

template <typename... Outer>
int fold_nested_sums(Outer... outers) {
    return (... + sum_inner(outers, 1, 2, 3));
}

template <typename... Outer>
int fold_multiplied(Outer... outers) {
    return (... + (outers * sum_inner(10, 20)));
}

struct Point3D {
    int x, y, z;
    Point3D(int x, int y, int z) : x(x), y(y), z(z) {}
    int sum() const {
        return x + y + z;
    }
};

template <typename... Points>
int fold_points(const Points&... pts) {
    return (... + pts.sum());
}

template <typename... Args>
void print_all_nested(Args... args) {
    ((printf("%d ", args)), ...);
    printf("\n");
}

int main() {
    printf("nested_sums: %d\n", fold_nested_sums(10, 20));
    printf("multiplied: %d\n", fold_multiplied(2, 3));
    Point3D p1(1, 2, 3);
    Point3D p2(4, 5, 6);
    printf("points_sum: %d\n", fold_points(p1, p2));
    print_all_nested(100, 200, 300);
    return 0;
}
