struct S { int f(int x = 5) { return x; } };
int main() {
  S s;
  int (S::*p)(int) = &S::f;
  return (s.*p)();   // defaults are not carried through a member pointer
}
