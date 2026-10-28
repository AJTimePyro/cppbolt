#include <cstdio>

struct Box {
    int value;
    Box() : value(0) {}
    Box(int v) : value(v) {}
    int& operator[](int) {
        printf("mut\n");
        return value;
    }
    const int& operator[](int) const {
        printf("ro\n");
        return value;
    }
};

int main() {
    Box b(11);
    const Box c(22);
    printf("%d\n", b[0]);
    printf("%d\n", c[0]);
    b[0] = 33;
    printf("%d\n", b[0]);
    const int& ref = c[0];
    printf("%d\n", ref);
    return 0;
}
