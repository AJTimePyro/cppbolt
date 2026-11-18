#include <cstdio>

struct Base1 {
    int b1_val;
    int get_b1() { return b1_val; }
};

struct Base2 {
    int b2_val;
    int get_b2() { return b2_val * 2; }
};

struct Derived : Base1, Base2 {
    int d_val;
};

int main() {
    Derived d;
    d.b1_val = 10;
    d.b2_val = 20;
    d.d_val = 30;

    int Base1::* pm_b1 = &Base1::b1_val;
    int Base2::* pm_b2 = &Base2::b2_val;

    int Derived::* pmd_b1 = pm_b1;
    int Derived::* pmd_b2 = pm_b2;

    std::printf("derived dot b1: %d\n", d.*pmd_b1);
    std::printf("derived dot b2: %d\n", d.*pmd_b2);

    Derived* pd = &d;
    std::printf("derived arrow b1: %d\n", pd->*pmd_b1);
    std::printf("derived arrow b2: %d\n", pd->*pmd_b2);

    int (Base2::* pm_get_b2)() = &Base2::get_b2;
    int (Derived::* pmd_get_b2)() = pm_get_b2;

    std::printf("call b2 via derived: %d\n", (d.*pmd_get_b2)());
    std::printf("call b2 via derived ptr: %d\n", (pd->*pmd_get_b2)());

    return 0;
}
