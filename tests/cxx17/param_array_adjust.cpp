#include <iostream>
struct B { virtual void f(int a[5]){ std::cout << "B" << a[0] << "\n"; } virtual ~B(){} };
struct D : B { void f(int* a) override { std::cout << "D" << a[0] << "\n"; } };
int main(){ int a[5]={4}; B* b = new D; b->f(a); delete b; }
