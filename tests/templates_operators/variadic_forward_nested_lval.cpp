#include <cstdio>
#include <utility>

void check(int& a) {
    a = 100;
}

void add_five(int& a) {
    a += 5;
}

template <typename... Args>
void outer(Args&&... args) {
    check(std::forward<Args>(args)...);
    add_five(std::forward<Args>(args)...);
}

int main() {
    int val = 42;
    outer(val);
    std::printf("val = %d\n", val);
    return 0;
}
