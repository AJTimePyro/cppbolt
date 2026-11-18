struct A { int a; };
struct B1 : A {};
struct B2 : A {};
struct D : B1, B2 {};
int main() {
    D d{};
    int A::* pa = &A::a;
    return d.*pa;
}
