#include <cstdio>
template<class T> void tlog(T v){ std::printf("t%d\n",(int)v); }
template<class T> struct B { T v; B(T x):v(x){} ~B(){ tlog(v); std::printf("~B\n"); } };
struct D : B<int> { D():B<int>(407){} };
int main(){ { D d; } }
