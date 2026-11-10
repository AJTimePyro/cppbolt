#include <cstdio>

void take_rref(int&& x) {
    printf("rref: %d\n", x);
}

int main() {
    take_rref(42);
    return 0;
}
