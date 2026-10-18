#include <cstdio>

int counter() {
    static int c = 10;
    c++;
    return c;
}

int main() {

    printf("c1 = %d\n", counter());
    printf("c2 = %d\n", counter());
    printf("c3 = %d\n", counter());
    return 0;
}
