#include <cstdio>

void test_switch(int val) {
    switch (val) {
        case 1:
            std::printf("case 1\n");
            break;
        case 2:
            std::printf("case 2 fallthrough\n");
            // fallthrough
        case 3:
            std::printf("case 2 or 3\n");
            break;
        default:
            std::printf("default: %d\n", val);
            break;
    }
}

int main() {
    test_switch(1);
    test_switch(2);
    test_switch(3);
    test_switch(42);
    return 0;
}
