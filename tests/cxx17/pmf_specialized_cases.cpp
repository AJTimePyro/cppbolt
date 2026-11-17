#include <cstdio>
template<class T> struct Base {
  int v;
  Base(int x):v(x){}
  int f(int x){ return v+x; }
  int g(){ return v; }
  virtual int vf(){ return v+10; }
};
using BI = Base<int>;

int main(){
  BI b(5);
  int (BI::*pf)(int) = &BI::f;
  int (BI::*pg)() = &BI::g;
  printf("%d %d\n", (b.*pf)(3), (b.*pg)());

  int (BI::*pvf)() = &BI::vf;
  printf("%d %d\n", (b.*pvf)(), ((&b)->*pvf)());

  int (BI::*pn)() = nullptr;
  printf("%d\n", pn==nullptr);
  return 0;
}
