struct B { virtual int f(int* p) { return *p; } };
struct D : B { int f(const int* p) override { return *p; } };
int main() { int x = 3; D d; return d.f(&x); }
