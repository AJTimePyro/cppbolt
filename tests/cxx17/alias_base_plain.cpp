#include <iostream>
struct Base { int x; Base(int v):x(v){} };
using Alias = Base;
struct Derived : Alias { Derived() : Alias(6) {} };
int main(){ Derived d; std::cout << d.x << "\n"; }
