#include <cstdio>
int g = 461;
template<class T> struct B { T v; B(T x):v(x){} ~B(){ std::printf("~B g=%d\n", g); } };
struct D : B<int> { D():B<int>(1){} };
int main(){ { D d; } }
