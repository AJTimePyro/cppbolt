#include <cstdio>

int f1();
using A = decltype(f1());
int f2();
using B = decltype(f2());

template <class T> T ft(T x) { return x; }
using C = decltype(ft(1));

namespace ns { int g(); }
using D = decltype(ns::g());
using ns::g;
using E = decltype(g());
using namespace ns;
using F = decltype(g());

int h(int x);
template <class T> using H = decltype(h(T()));
using I = H<int>;

int f1() { return 1; }
int f2() { return 2; }
namespace ns { int g() { return 3; } }
int h(int x) { return x; }

int main() {
  A a = f1();
  B b = f2();
  C c = ft(4);
  D d = ns::g();
  E e = g();
  F f = g();
  I i = h(7);
  printf("%d\n", a + b + c + d + e + f + i);
  return 0;
}
