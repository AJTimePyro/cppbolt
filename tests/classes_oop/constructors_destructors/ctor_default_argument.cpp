#include <cstdio>

class Scale {
    int value;
public:
    Scale(int base, int factor = 7) : value(base * factor) {
        std::printf("scale %d\n", value);
    }
    int get() const { return value; }
};

class Multi {
    int a;
    int b;
    int c;
public:
    Multi(int x, int y = 2, int z = 5) : a(x), b(y), c(z) {
        std::printf("multi %d %d %d\n", a, b, c);
    }
    int sum() const { return a + b + c; }
};

class Base {
public:
    Base(int a, int b = 3) {
        std::printf("base %d %d\n", a, b);
    }
    virtual ~Base() {}
};

class Derived : public Base {
public:
    Derived(int a) : Base(a) {
        std::printf("derived\n");
    }
};

int main() {
    Scale s(3);
    std::printf("s=%d\n", s.get());
    Multi m(1);
    std::printf("m=%d\n", m.sum());
    Multi n(1, 20);
    std::printf("n=%d\n", n.sum());
    Derived d(4);
    Scale* p = new Scale(2);
    std::printf("p=%d\n", p->get());
    delete p;
    return 0;
}
