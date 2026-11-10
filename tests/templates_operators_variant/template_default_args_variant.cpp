#include <iostream>

template <typename A, typename B = double, typename C = int>
struct TripleDef {
    A a;
    B b;
    C c;
    TripleDef(A x, B y, C z) : a(x), b(y), c(z) {}
    void report() {
        std::cout << a << " " << static_cast<int>(b) << " " << c << std::endl;
    }
};

int main() {
    TripleDef<int> t1(10, 20.5, 30);
    t1.report();
    TripleDef<const char*, int> t2("val", 99, 100);
    t2.report();
    return 0;
}
