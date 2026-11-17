#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} ~Base(){ std::cout << "~Base "; } };
using Alias = Base<int>;
struct D : Alias { D():Alias(1){} };
int main(){ { D d; } std::cout << "\n"; }
