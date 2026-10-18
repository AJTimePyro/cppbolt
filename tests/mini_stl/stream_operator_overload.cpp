#include <iostream>

struct Point2D {
    int x;
    int y;
    Point2D(int x_ = 0, int y_ = 0) : x(x_), y(y_) {}
};

std::ostream& operator<<(std::ostream& os, const Point2D& pt) {
    os << "(" << pt.x << ", " << pt.y << ")";
    return os;
}

int main() {
    Point2D p1(12, 34);
    Point2D p2(56, 78);
    std::cout << "p1=" << p1 << " and p2=" << p2 << std::endl;
    return 0;
}
