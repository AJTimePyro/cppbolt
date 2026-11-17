struct B { virtual int f() & { return 1; } };
struct D : B { int f() && override { return 2; } };
int main() { return 0; }
