#include <cstdio>

class Base {
public:
    int value;
    Base() : value(10) {}
};

class Derived : public Base {
public:
    int value;
    Derived() : value(20) {}
};

int main() {
    Derived derived;
    Base& base = derived;
    std::printf("%d %d\n", derived.value, base.value);
    derived.value = 25;
    std::printf("%d %d\n", derived.value, base.value);
    return 0;
}
