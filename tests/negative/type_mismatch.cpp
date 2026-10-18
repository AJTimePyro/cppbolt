struct A { int x; };
struct B { int y; };

int main() {
    A a;
    B b = a; // error: no matching conversion
    return 0;
}
