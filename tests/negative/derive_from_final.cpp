struct B { virtual int f() { return 1; } };
struct D final : B { int f() override { return 2; } };
struct E : D { int f() override { return 3; } };
int main() { E e; return e.f(); }
