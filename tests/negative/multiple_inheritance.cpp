#include <cstdio>

struct A { virtual void f(); };
struct B { virtual void g(); };
struct C : A, B { int c; };

int main() { return 0; }
