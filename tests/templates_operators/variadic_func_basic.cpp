#include <cstdio>

void print_all() {
    printf("\n");
}

template <typename T, typename... Rest>
void print_all(T first, Rest... rest) {
    printf("%d ", (int)first);
    print_all(rest...);
}

int main() {
    print_all(10, 20, 30);
    return 0;
}
