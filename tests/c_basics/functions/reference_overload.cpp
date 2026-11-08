#include <cstdio>

void f(int& x) {
    std::printf("lvalue\n");
}

void f(const int& x) {
    std::printf("rvalue\n");
}

int main() {
    int a = 10;
    f(a);
    f(42);
    return 0;
}
