#include <cstdio>
struct A { virtual ~A(){} virtual A* f(){return this;} };
struct B { virtual ~B(){} virtual B* g(){return this;} };
struct D : A, B { D* f() override {return this;} D* g() override {return this;} };
int main(){
  D d;
  A* a=&d; B* b=&d;
  printf("%d %d\n", a->f()==a, b->g()==b);
  return 0;
}
