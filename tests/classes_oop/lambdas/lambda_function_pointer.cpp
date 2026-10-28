#include <stdio.h>

int apply(int (*fn)(int), int x) {
    return fn(x);
}

int main() {
    // 1. Initializing function pointer variable directly from prvalue lambda
    int (*fp1)(int) = [](int x) {
        return x * 2;
    };
    printf("fp1(5) = %d\n", fp1(5));

    // 2. Passing captureless lambda directly as function pointer argument
    int res = apply([](int x) {
        return x + 10;
    }, 20);
    printf("apply = %d\n", res);

    // 3. Assigning stored captureless lambda object to function pointer
    auto l = [](int x) {
        return x - 1;
    };
    int (*fp2)(int) = l;
    printf("fp2(8) = %d\n", fp2(8));

    return 0;
}
