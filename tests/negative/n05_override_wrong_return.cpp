struct B { virtual int f() { return 1; } };
struct D : B { double f() override { return 2.0; } };
int main() { D d; return (int)d.f(); }
