#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){} virtual T get(){return x;} };
namespace ns { using Alias = Base<int>; }
struct Derived : ns::Alias { Derived() : ns::Alias(7) {} int get() override {return x+1;} };
int main(){ Derived d; std::cout << d.x << " " << d.get() << "\n"; }
