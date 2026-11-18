struct B { virtual int f(int) { return 1; } };
struct D : B { int f(double) override { return 2; } };
int main() { D d; return d.f(1); }
