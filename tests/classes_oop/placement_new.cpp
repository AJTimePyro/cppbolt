#include <cstdio>
#include <new>

struct Point {
    int x;
    int y;
    Point(int x, int y) : x(x), y(y) {
        printf("Point ctor: %d, %d\n", x, y);
    }
    ~Point() {
        printf("Point dtor: %d, %d\n", x, y);
    }
};

char buffer[128];

int main() {
    Point* p = new (buffer) Point(10, 20);
    printf("p->x = %d, p->y = %d\n", p->x, p->y);
    return 0;
}
