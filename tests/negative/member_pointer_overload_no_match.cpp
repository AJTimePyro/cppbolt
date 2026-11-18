struct C {
    int f(int x) { return x; }
    int f(double x) { return static_cast<int>(x); }
};

int main() {
    int (C::*pmf)(char) = &C::f;
    return 0;
}
