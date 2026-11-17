#include <cstdio>
struct B { virtual ~B() {} virtual int f() { return 1; } };
struct D final : B { int f() override { return 2; } };
int main() { D d; B* b = &d; std::printf("%d\n", b->f()); return 0; }
