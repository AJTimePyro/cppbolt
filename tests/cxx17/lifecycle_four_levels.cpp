#include <cstdio>
struct R { int v; R(int x):v(x){printf("R%d+",v);} R(const R& o):v(o.v){printf("R%d+",v);} ~R(){printf("R%d-",v);} };
struct A { virtual ~A(){printf("A-");} R r; A():r(1){printf("A+");} A(const A& o):r(o.r){printf("A+");} };
struct B : A { virtual ~B(){printf("B-");} R r; B():r(2){printf("B+");} B(const B& o):A(o),r(o.r){printf("B+");} };
struct C : B { virtual ~C(){printf("C-");} R r; C():r(3){printf("C+");} C(const C& o):B(o),r(o.r){printf("C+");} };
struct D : C { ~D() override {printf("D-");} R r; D():r(4){printf("D+");} D(const D& o):C(o),r(o.r){printf("D+");} };
int main(){
  printf("[copy] "); { D d; D e=d; } printf("\n");
  printf("[A*] "); { A* p=new D; delete p; } printf("\n");
  printf("[B*] "); { B* p=new D; delete p; } printf("\n");
  printf("[C*] "); { C* p=new D; delete p; } printf("\n");
  printf("[D*] "); { D* p=new D; delete p; } printf("\n");
  return 0;
}
