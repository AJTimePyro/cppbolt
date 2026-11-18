#include <cstdio>
void later(int);
template<class T> struct B { T v; B(T x):v(x){} ~B(){ later((int)v); std::printf("~B\n"); } };
void later(int x){ std::printf("later%d\n", x); }
struct D : B<int> { D():B<int>(462){} };
int main(){ { D d; } }
