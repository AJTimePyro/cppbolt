#include <cstdio>
void later();
template<class T> struct B { T v; B(T x):v(x){} ~B(){ later(); int* p = v; (void)p; } };
void later(){ std::printf("later\n"); }
int main(){ B<int> b(1); }
