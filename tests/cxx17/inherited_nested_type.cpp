#include <cstdio>

struct Base { typedef int value_type; };
struct Derived : Base {};
using A = Derived::value_type;

struct Hider : Base { typedef double value_type; };
using B = Hider::value_type;

struct A1 { typedef int tag; };
struct B1 { typedef int other; };
struct Multi : A1, B1 {};
using C = Multi::tag;
using D = Multi::other;

template <class T> struct TBase { typedef T value_type; };
struct TDerived : TBase<int> {};
using E = TDerived::value_type;

int main() {
  A a = 1; B b = 2.5; C c = 3; D d = 4; E e = 5;
  printf("%d %d %d %d %d\n", (int)sizeof(A), (int)sizeof(B), c, d, e);
  (void)a; (void)b;
  return 0;
}
