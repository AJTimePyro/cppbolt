#include <cstdio>
template<class T> struct Base {
  virtual ~Base(){} virtual Base* clone(){return new Base();} T v;
};
using BaseI = Base<int>;
struct Derived : BaseI {
  Derived* clone() override {return new Derived();}
};
int main(){
  BaseI* b=new Derived;
  BaseI* c=b->clone();
  printf("ptr=%d id=%d\n", c!=nullptr, c==b);
  delete c; delete b;
  return 0;
}
