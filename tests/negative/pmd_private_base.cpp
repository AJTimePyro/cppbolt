struct A { int a; };
struct D : private A { int d; };
int main() {
  int A::* pa = &A::a;
  int D::* pd = pa;   // A is an inaccessible base of D here
  (void)pd;
  return 0;
}
