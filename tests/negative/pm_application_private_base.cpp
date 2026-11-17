struct A { int a; };
struct D : private A {};
int main() {
    D d{};
    int A::* pa = &A::a;
    return d.*pa;
}
