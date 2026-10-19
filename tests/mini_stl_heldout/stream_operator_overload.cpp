#include <iostream>

struct Vec3 {
    int i;
    int j;
    int k;
    Vec3(int a = 0, int b = 0, int c = 0) : i(a), j(b), k(c) {}
};

std::ostream& operator<<(std::ostream& os, const Vec3& v) {
    os << "<" << v.i << ";" << v.j << ";" << v.k << ">";
    return os;
}

int main() {
    Vec3 a(1, 2, 3);
    Vec3 b(-4, 5, 0);
    std::cout << "a=" << a << " b=" << b << std::endl;
    return 0;
}
