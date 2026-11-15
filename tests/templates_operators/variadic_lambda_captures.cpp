#include <cstdio>

template <typename... Args>
void test_capture_by_value(Args... args) {
    auto l = [args...]() {
        int sum = 0;
        int dummy[] = { (sum += args, 0)... };
        (void)dummy;
        return sum;
    };
    std::printf("val: %d\n", l());
}

template <typename... Args>
void test_capture_by_ref(Args... args) {
    auto l = [&args...]() {
        int sum = 0;
        int dummy[] = { (sum += args, 0)... };
        (void)dummy;
        return sum;
    };
    std::printf("ref: %d\n", l());
}

int main() {
    test_capture_by_value(1, 2, 3, 4);
    test_capture_by_ref(10, 20, 30);
    return 0;
}
