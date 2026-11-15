#include <stdio.h>

void side_effect(int x) {
    printf("swallowed: %d\n", x);
}

template <typename... Ts>
void swallow(Ts... args) {
    int dummy[] = { (side_effect(args), 0)... };
    (void)dummy;
}

int main() {
    swallow(10, 20, 30);
    return 0;
}
