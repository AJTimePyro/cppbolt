#include <memory>

std::unique_ptr<int> test(std::unique_ptr<int>& ref) {
    // Returning a reference parameter cannot implicitly move.
    // It must not copy either because unique_ptr is move-only.
    return ref;
}

int main() {
    return 0;
}
