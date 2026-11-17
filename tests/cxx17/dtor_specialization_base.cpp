#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} ~Base(){ std::cout << "~Base "; } };
struct D : Base<int> { D():Base<int>(1){} ~D(){ std::cout << "~D "; } };
int main(){ { D d; } std::cout << "\n"; }
