#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} ~Base(){ std::cout << "~Base "; } };
using Alias = Base<int>;
int main(){ { Alias a(1); } std::cout << "\n"; }
