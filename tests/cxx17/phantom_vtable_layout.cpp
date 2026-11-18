#include <cstdio>
struct A { virtual ~A(){} virtual int f(){return 1;} int a; A():a(4){} };
struct Pad { long long x; Pad():x(0x1122334455667788LL){} };

// polymorphic base second, derived adds an override
struct Mid : Pad, A { int m; Mid():m(5){} };
struct Leaf : Mid { int f() override {return 9;} };

// polymorphic base first, non-polymorphic pad after
struct Mid2 : A, Pad { int m; Mid2():m(6){} };
struct Leaf2 : Mid2 { int f() override {return 10;} };

// independent secondary polymorphic base
struct Other { virtual ~Other(){} virtual int g(){return 2;} };
struct MI : Pad, A, Other { int f() override {return 11;} int g() override {return 12;} };

int main(){
  Leaf l; Mid* m=&l; A* a=&l;
  printf("%d %d %d x=%lld\n", m->f(), a->f(), l.f(), l.x);
  int (A::*pf)()=&A::f;
  printf("pmf: %d %d\n", (m->*pf)(), (a->*pf)());

  Leaf2 l2; A* a2=&l2;
  printf("%d %d x=%lld\n", a2->f(), l2.f(), l2.x);

  MI mi; A* ma=&mi; Other* mo=&mi;
  printf("%d %d %d x=%lld\n", ma->f(), mo->g(), mi.f(), mi.x);
  return 0;
}
