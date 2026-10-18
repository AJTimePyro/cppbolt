#include <cstdio>

template <typename T1, typename T2>
void print_pair(T1 a, T2 b) {
    std::printf("pair: %d, %d\n", (int)a, (int)b);
}

int main() {
    print_pair(10, 20);
    print_pair('A', 66);
    return 0;
}
