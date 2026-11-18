#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){ std::cout << "~Base "; } };
using Alias = Base<int>;
int main(){ Alias* p=new Alias(1); delete p; std::cout << "\n"; }
