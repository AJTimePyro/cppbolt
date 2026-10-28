#include <cstdio>

struct Point {
    int x;
    int y;

    int sum() {
        return x + y;
    }

    void translate(int dx, int dy) {
        x += dx;
        y += dy;
    }
};

int main() {
    Point p;
    p.x = 10;
    p.y = 20;
    std::printf("Point sum: %d\n", p.sum());
    p.translate(5, -2);
    std::printf("Translated: (%d, %d)\n", p.x, p.y);
    return 0;
}
