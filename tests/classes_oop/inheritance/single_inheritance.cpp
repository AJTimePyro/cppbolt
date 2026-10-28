#include <cstdio>

class Base {
protected:
    int base_val;
public:
    Base(int v) : base_val(v) {
        std::printf("Base ctor %d\n", base_val);
    }
    ~Base() {
        std::printf("Base dtor %d\n", base_val);
    }
    void print_base() {
        std::printf("base_val: %d\n", base_val);
    }
};

class Derived : public Base {
    int derived_val;
public:
    Derived(int b, int d) : Base(b), derived_val(d) {
        std::printf("Derived ctor %d\n", derived_val);
    }
    ~Derived() {
        std::printf("Derived dtor %d\n", derived_val);
    }
    void print_all() {
        print_base();
        std::printf("derived_val: %d\n", derived_val);
    }
};

int main() {
    Derived d(10, 20);
    d.print_all();
    return 0;
}
