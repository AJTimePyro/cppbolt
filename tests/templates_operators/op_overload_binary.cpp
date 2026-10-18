#include <cstdio>

class Vector2 {
public:
    int x, y;
    Vector2(int x_ = 0, int y_ = 0) : x(x_), y(y_) {}

    Vector2 operator+(const Vector2& o) const {
        return Vector2(x + o.x, y + o.y);
    }
    Vector2 operator-(const Vector2& o) const {
        return Vector2(x - o.x, y - o.y);
    }
    bool operator==(const Vector2& o) const {
        return x == o.x && y == o.y;
    }
};

int main() {
    Vector2 v1(2, 5);
    Vector2 v2(3, 1);
    Vector2 sum = v1 + v2;
    Vector2 diff = v1 - v2;

    std::printf("sum: (%d, %d)\n", sum.x, sum.y);
    std::printf("diff: (%d, %d)\n", diff.x, diff.y);
    std::printf("eq: %d\n", (sum == Vector2(5, 6)) ? 1 : 0);
    return 0;
}
