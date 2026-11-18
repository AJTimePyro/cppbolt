#include <cstdio>
int g = 7;
template<class T> struct Inner { ~Inner(){ std::printf("~Inner g=%d\n", g); } };
template<class T> struct Outer { Inner<T> i; ~Outer(){ std::printf("~Outer g=%d\n", g); } };
int main(){ { Outer<int> o; } }
