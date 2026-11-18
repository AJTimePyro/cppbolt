struct A { int a; };
struct B : A { int b; };
struct C : A { int c; };
struct D : B, C { int d; };
int main() {
  int A::* pa = &A::a;
  int D::* pd = pa;   // A is an ambiguous base of D
  (void)pd;
  return 0;
}
