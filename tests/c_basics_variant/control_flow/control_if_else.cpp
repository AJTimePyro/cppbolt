#include <cstdio>

void test_branch(int n) {
    if (n > 50) {
        std::printf("gt50: %d\n", n);
    } else if (n > 20) {
        std::printf("gt20: %d\n", n);
    } else if (n > 0) {
        std::printf("pos: %d\n", n);
    } else if (n == 0) {
        std::printf("zero\n");
    } else {
        std::printf("neg: %d\n", n);
    }
}

int main() {
    test_branch(75);
    test_branch(35);
    test_branch(12);
    test_branch(0);
    test_branch(-18);
    int t = 5 > 3 ? 99 : 88;
    std::printf("ternary: %d\n", t);
    return 0;
}
