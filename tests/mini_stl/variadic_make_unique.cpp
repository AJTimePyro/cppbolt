#include <cstdio>
#include <memory>

struct Widget {
    int a, b, c, d;
    Widget(int a, int b, int c, int d) : a(a), b(b), c(c), d(d) {}
    void print() { printf("%d %d %d %d\n", a, b, c, d); }
};

int main() {
    auto w = std::make_unique<Widget>(1, 2, 3, 4); // 4 args! previously limited to 3
    w->print();
    return 0;
}
