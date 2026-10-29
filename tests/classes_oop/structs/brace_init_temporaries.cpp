#include <cstdio>

struct Point {
    int x;
    int y;
};

struct Value {
    int v;
    int get() { return v; }
};

Point make_point(int a, int b) {
    return {a, b};
}

int consume(Point p) {
    return p.x + p.y;
}

int consume_ref(const Point& p) {
    return p.x * p.y;
}

int main() {
    int v1 = Value{42}.get();
    std::printf("v1: %d\n", v1);

    int px = Point{10, 20}.x;
    std::printf("px: %d\n", px);

    Point p2 = make_point(30, 40);
    std::printf("p2: %d %d\n", p2.x, p2.y);

    int sum = consume({5, 15});
    std::printf("sum: %d\n", sum);

    int prod = consume_ref({6, 7});
    std::printf("prod: %d\n", prod);

    Point p{1, 1};
    p = {100, 200};
    std::printf("p: %d %d\n", p.x, p.y);

    const Point& r = Point{8, 9};
    std::printf("r: %d %d\n", r.x, r.y);

    return 0;
}
