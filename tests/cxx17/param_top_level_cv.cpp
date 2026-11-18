#include <iostream>
struct B { virtual void f(int x){ std::cout << "B" << x << "\n"; } virtual ~B(){} };
struct D : B { void f(const int x) override { std::cout << "D" << x << "\n"; } };
int main(){ B* b = new D; b->f(9); delete b; }
