#include <cstdio>

struct S {
    int mut() { return 1; }
    int con() const { return 2; }
};

struct A { int a; };
struct D : private A {
    D() { a = 7; }
    int read() const {
        // A is a private base, but this is inside the derived class.
        int A::* pa = &A::a;
        return this->*pa;
    }
};

int main() {
    const S cs{};
    int (S::* pc)() const = &S::con;
    std::printf("%d %d\n", (cs.*pc)(), (cs.*pc)());
    D d{};
    std::printf("%d\n", d.read());
    return 0;
}
