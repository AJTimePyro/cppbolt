struct Base { int f() { return 1; } };
struct Derived : Base {
    int f() override { return 2; }
};
int main() { Derived d; return d.f(); }
