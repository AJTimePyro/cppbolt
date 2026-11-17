#include <iostream>
template <typename T> struct B { T v; B(T x):v(x){} virtual ~B(){} };
using BI = B<int>;
using BD = B<double>;
struct D : BI, BD { D():BI(1),BD(2.5){} };
int main(){ D d; BI* a=&d; BD* b=&d; std::cout << a->v << " " << b->v << "\n"; }
