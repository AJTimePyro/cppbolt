#include <cstdio>
#include <memory>

struct Point5D {
    int v[5];
    Point5D(int a, int b, int c, int d, int e) {
        v[0] = a; v[1] = b; v[2] = c; v[3] = d; v[4] = e;
    }
    void print() const {
        printf("%d %d %d %d %d\n", v[0], v[1], v[2], v[3], v[4]);
    }
};

int main() {
    auto p = std::make_unique<Point5D>(10, 20, 30, 40, 50);
    p->print();
    return 0;
}
