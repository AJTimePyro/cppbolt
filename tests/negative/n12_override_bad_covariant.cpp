struct A {}; struct B {};
struct Base { virtual A* f() { return nullptr; } };
struct D : Base { B* f() override { return nullptr; } };
int main() { return 0; }
