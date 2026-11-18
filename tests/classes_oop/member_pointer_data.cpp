#include <cstdio>

struct Point {
    int x;
    int y;
};

int main() {
    Point pt;
    pt.x = 10;
    pt.y = 20;

    int Point::* px = &Point::x;
    int Point::* py = &Point::y;
    int Point::* pnull = nullptr;

    std::printf("pt.*px = %d\n", pt.*px);
    std::printf("pt.*py = %d\n", pt.*py);

    Point* p = &pt;
    std::printf("p->*px = %d\n", p->*px);
    std::printf("p->*py = %d\n", p->*py);

    p->*px = 100;
    pt.*py = 200;
    std::printf("modified: x=%d y=%d\n", pt.x, pt.y);

    if (pnull == nullptr) {
        std::printf("pnull is null\n");
    }
    if (px != nullptr) {
        std::printf("px is not null\n");
    }
    return 0;
}
