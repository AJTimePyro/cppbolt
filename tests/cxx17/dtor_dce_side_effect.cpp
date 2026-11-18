#include <cstdio>
int g=0;
template<class T> struct B { T v; B(T x):v(x){} ~B(){ g=(int)v; } };
int main(){ B<int>* p=new B<int>(481); delete p; std::printf("g=%d\n", g); }
