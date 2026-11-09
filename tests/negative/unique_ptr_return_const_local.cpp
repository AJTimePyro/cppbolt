#include <memory>

std::unique_ptr<int> test() {
    const std::unique_ptr<int> p(new int(10));
    // A const local object cannot be moved from (move constructor requires non-const rvalue reference).
    // It must not copy either.
    return p;
}

int main() {
    test();
    return 0;
}
