#include <cstdio>

typedef int T;
typedef double D;

int main() {
    T a = T();
    T b = T(42);
    int c = int();
    int d = int(100);
    D e = D();
    std::printf("%d %d %d %d %.1f\n", a, b, c, d, e);
    return 0;
}
