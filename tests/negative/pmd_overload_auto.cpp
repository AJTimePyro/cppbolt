struct S { int f() { return 1; } int f() const { return 2; } };
int main() {
  auto p = &S::f;   // overload set cannot initialize auto
  (void)p;
  return 0;
}
