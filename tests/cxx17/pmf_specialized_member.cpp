#include <cstdio>
template<class T> struct S { int f(int x){ return x+T(1); } int g(){return 7;} };
int main(){
  S<int> s;
  auto p = &S<int>::f;
  printf("%d\n", (s.*p)(5));
  S<int>* ptr=&s;
  printf("%d\n", (ptr->*p)(6));
  return 0;
}
