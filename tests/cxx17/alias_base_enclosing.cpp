#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){} };
struct Outer {
  using Alias = Base<int>;
  struct Inner : Alias { Inner() : Alias(8) {} };
};
int main(){ Outer::Inner d; std::cout << d.x << "\n"; }
