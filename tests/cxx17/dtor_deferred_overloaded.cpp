#include <cstdio>
void ov(int){ std::printf("ovi\n"); }
void ov(double){ std::printf("ovd\n"); }
template<class T> struct B { T v; B(T x):v(x){} ~B(){ ov(v); std::printf("~B\n"); } };
struct D : B<double> { D():B<double>(406.5){} };
int main(){ { D d; } }
