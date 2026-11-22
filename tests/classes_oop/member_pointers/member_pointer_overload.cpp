#include <cstdio>

struct Widget {
    int pick(int x) { return x + 1; }
    int pick(char x) { return x + 2; }
};

struct Mixed {
    int call(int x) { return x + 10; }
    int call(double x) { return static_cast<int>(x) + 20; }
};

struct Base {
    int base_only(int x) { return x + 100; }
    int base_only(char x) { return x + 200; }
};

struct Pad {
    int padding;
};

struct Derived : Pad, Base {
    int own;
};

int main() {
    Widget w;
    int (Widget::*same_abi)(int) = &Widget::pick;
    std::printf("same abi: %d\n", (w.*same_abi)(5));

    Mixed m;
    int (Mixed::*as_int)(int) = &Mixed::call;
    int (Mixed::*as_double)(double) = &Mixed::call;
    std::printf("int: %d\n", (m.*as_int)(7));
    std::printf("double: %d\n", (m.*as_double)(7));
    Mixed* pm = &m;
    std::printf("arrow int: %d\n", (pm->*as_int)(1));
    std::printf("arrow double: %d\n", (pm->*as_double)(1));

    Derived d;
    int (Base::*base_pmf)(int) = &Base::base_only;
    int (Derived::*derived_pmf)(int) = base_pmf;
    std::printf("multi inheritance: %d\n", (d.*derived_pmf)(5));

    return 0;
}
