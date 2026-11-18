#include <iostream>
struct A { virtual ~A(){} virtual void f(){ std::cout<<"Af"; } };
struct B { virtual ~B(){} virtual void g(){ std::cout<<"Bg"; } };
struct C : A,B { void f() override { std::cout<<"Cf"; } void g() override { std::cout<<"Cg"; } };
int main(){ C c; A& a=c; B& b=c; a.f(); b.g(); std::cout<<"\n"; }
