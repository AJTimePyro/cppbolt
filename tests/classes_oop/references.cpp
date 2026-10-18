#include <cstdio>

void swap(int& a, int& b) {
    int tmp = a;
    a = b;
    b = tmp;
}

int global_val = 10;
int& get_ref() {
    return global_val;
}

int main() {
    int x = 100;
    int y = 200;
    swap(x, y);
    std::printf("swapped: x=%d, y=%d\n", x, y);

    get_ref() = 999;
    std::printf("global_val: %d\n", global_val);
    return 0;
}
