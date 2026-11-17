#include <iostream>
using CI = const int;
struct B { virtual void f(CI x){ std::cout << "B" << x << "\n"; } virtual ~B(){} };
struct D : B { void f(int x) override { std::cout << "D" << x << "\n"; } };
int main(){ B* b = new D; b->f(9); delete b; }
