#include <stdio.h>

int main() {
    int x = 42;
    auto outer = [=]() {
        auto inner = [x]() {
            return x;
        };
        return inner();
    };
    printf("%d\n", outer());
    return 0;
}
