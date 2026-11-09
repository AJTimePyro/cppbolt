#include <cstdio>

struct Defaults {
    int a = 10;
    int b{20};
    int c = 30;
};

struct CustomCtor {
    int x = 100;
    int y = 200;
    CustomCtor() {}
    CustomCtor(int val) : x(val) {}
};

template<typename T>
struct Box {
    T val = 42;
};

int main() {
    Defaults d;
    std::printf("d: %d %d %d\n", d.a, d.b, d.c);

    CustomCtor c1;
    CustomCtor c2(55);
    std::printf("c1: %d %d\n", c1.x, c1.y);
    std::printf("c2: %d %d\n", c2.x, c2.y);

    Box<int> b;
    std::printf("b: %d\n", b.val);

    return 0;
}
