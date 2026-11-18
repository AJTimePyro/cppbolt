struct B { virtual int f() const { return 1; } };
struct D : B { int f() override { return 2; } };
int main() { D d; return d.f(); }
