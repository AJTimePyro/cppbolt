struct B { int g(int) { return 1; } };
struct D : B { int g(double) { return 2; } };
int main() {
  int (D::*p)(int) = &D::g;   // D::g(double) hides B::g(int)
  (void)p;
  return 0;
}
