#include <cstdio>
struct Resource {
  int v;
  Resource(int x):v(x){ printf("R%d+",v); }
  Resource(const Resource& o):v(o.v){ printf("R%d+",v); }
  ~Resource(){ printf("R%d-",v); }
  Resource& operator=(const Resource& o){ v=o.v; return *this; }
};
struct A { Resource r; A():r(1){printf("A+");} A(const A& o):r(o.r){printf("A+");} ~A(){printf("A-");} };
struct B : A { Resource r; B():r(2){printf("B+");} B(const B& o):A(o),r(o.r){printf("B+");} ~B(){printf("B-");} };
struct C : B { Resource r; C():r(3){printf("C+");} C(const C& o):B(o),r(o.r){printf("C+");} ~C(){printf("C-");} };
int main(){
  printf("[stack] "); { C c; } printf("\n");
  printf("[copy] "); { C c; C d=c; } printf("\n");
  printf("[delete] "); { C* p=new C; delete p; } printf("\n");
  return 0;
}
