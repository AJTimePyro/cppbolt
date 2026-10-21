#include <cstdio>

struct A {};
struct B {};
struct C : A, public B {};

int main() { return 0; }
