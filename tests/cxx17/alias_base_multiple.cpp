#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){} };
using A1 = Base<int>;
using A2 = Base<int>;
struct D1 : A1 { D1():A1(3){} };
struct D2 : A2 { D2():A2(4){} };
int main(){ D1 a; D2 b; std::cout << a.x << " " << b.x << "\n"; }
