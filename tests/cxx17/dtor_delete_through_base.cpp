#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){ std::cout << "~Base "; } };
using Alias = Base<int>;
struct D : Alias { D():Alias(3){} ~D(){ std::cout << "~D "; } };
int main(){ Alias* p = new D; delete p; std::cout << "\n"; }
