#include <cstdio>

struct Point {
    int x;
    int y;
    int tag;

    Point(int x_val, int y_val) {
        x = x_val;
        y = y_val;
        tag = 100;
        std::printf("target ctor: (%d, %d)\n", x, y);
    }

    Point(int val) : Point(val, val * 2) {
        tag += 1;
        std::printf("delegating ctor: tag=%d\n", tag);
    }

    Point() : Point(5) {
        tag += 2;
        std::printf("default delegating ctor: tag=%d\n", tag);
    }
};

int main() {
    Point p;
    std::printf("final: x=%d y=%d tag=%d\n", p.x, p.y, p.tag);
    return 0;
}
