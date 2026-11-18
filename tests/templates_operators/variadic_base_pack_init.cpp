#include <stdio.h>

struct Base1 {
    int val;
    Base1(int v) : val(v) {}
};

template <typename... Bases>
struct Derived : Bases... {
    Derived(int x) : Bases(x)... {}
};

int main() {
    Derived<Base1> d(42);
    printf("Base pack init: %d\n", d.val);
    return 0;
}
