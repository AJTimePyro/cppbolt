#include <stdio.h>

struct Point {
    int x;
    int y;
    Point(int x_, int y_) : x(x_), y(y_) {}
    int sum() const { return x + y; }
};

struct PtrWrapper {
    Point* ptr;
    PtrWrapper(Point* p) : ptr(p) {}

    Point& operator*() {
        return *ptr;
    }

    Point* operator->() {
        return ptr;
    }
};

void inspect(PtrWrapper& ref) {
    printf("%d\n", ref->y);
    printf("%d\n", ref->sum());
}

int main() {
    Point pt(10, 20);
    PtrWrapper wrap(&pt);

    // Test operator*() lvalue assignment
    (*wrap).x = 42;
    printf("%d\n", (*wrap).x); // 42

    // Test operator-> field access
    printf("%d\n", wrap->y); // 20

    // Test operator-> method call
    printf("%d\n", wrap->sum()); // 62

    // Test operator-> on reference
    inspect(wrap);
    return 0;
}
