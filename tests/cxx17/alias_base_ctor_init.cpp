#include <iostream>
template <typename T> struct Base {
    T x;
    Base(T v) : x(v) {}
    virtual T get() { return x; }
    virtual ~Base() {}
};
using Alias = Base<int>;
struct Derived : Alias {
    Derived() : Alias(42) {}
    int get() override { return this->x + 1; }
};
int main() {
    Derived d;
    std::cout << d.x << "\n";
    std::cout << d.get() << "\n";
}
