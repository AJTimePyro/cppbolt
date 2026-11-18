#include <cstdio>

struct Base {
    int f() { return 1; }
};

struct Derived : Base {
    double f() { return 2.5; }
};

int main() {
    Derived d;
    std::printf("%.1f\n", d.f());
    return 0;
}
