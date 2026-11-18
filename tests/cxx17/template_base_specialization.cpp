#include <iostream>
template<class T> struct B { virtual ~B(){} virtual void f(T){std::cout<<"B";} };
struct D : B<int> { void f(int) override {std::cout<<"D";} };
int main(){ D d; B<int>& b=d; b.f(1); }
