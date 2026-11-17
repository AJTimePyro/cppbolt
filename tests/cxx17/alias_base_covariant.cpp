#include <iostream>
template <typename T> struct Base { T x; Base(T v):x(v){} virtual ~Base(){} virtual Base* self(){return this;} };
using Alias = Base<int>;
struct D : Alias { D():Alias(11){} D* self() override {return this;} };
int main(){ Alias* b=new D; Alias* r=b->self(); std::cout << r->x << " " << (r==b) << "\n"; delete b; }
