struct Base1 { int a; };
struct Unrelated { int b; };

int main() {
    Unrelated u;
    Base1* b = static_cast<Base1*>(&u);
    return 0;
}
