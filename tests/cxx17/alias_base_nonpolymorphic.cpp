#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} };
using Alias = Base<int>;
struct Derived : Alias { Derived() : Alias(5) {} };
int main(){ Derived d; std::cout << d.x << "\n"; }
