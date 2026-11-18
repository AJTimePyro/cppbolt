struct C {
    int f() { return 1; }
};

int main() {
    C c;
    int (C::*pmf)() = &C::f;
    int x = c.*pmf;
    return x;
}
