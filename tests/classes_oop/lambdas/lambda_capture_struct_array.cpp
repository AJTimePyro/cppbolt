#include <stdio.h>

struct Point {
    int x;
    int y;
    Point() : x(10), y(20) {}
};

int main() {
    Point pts[2];
    pts[0].x = 100;
    pts[1].y = 200;

    auto l = [pts]() {
        printf("%d %d\n", pts[0].x, pts[1].y);
    };

    l();
    return 0;
}
