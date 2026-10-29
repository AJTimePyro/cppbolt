#include <cstdio>

struct Point {
    int x;
    int y;
};

int main() {
    Point* p = new Point{10, 20};
    std::printf("p: %d %d\n", p->x, p->y);
    delete p;

    int* v = new int{99};
    std::printf("v: %d\n", *v);
    delete v;

    int* arr = new int[5]{1, 2, 3};
    for (int i = 0; i < 5; ++i) {
        std::printf("arr[%d] = %d\n", i, arr[i]);
    }
    delete[] arr;

    Point* arr2 = new Point[4]{Point{1, 2}, Point{3, 4}};
    for (int i = 0; i < 4; ++i) {
        std::printf("arr2[%d] = (%d, %d)\n", i, arr2[i].x, arr2[i].y);
    }
    delete[] arr2;

    return 0;
}
