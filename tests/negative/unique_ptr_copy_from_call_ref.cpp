#include <memory>

std::unique_ptr<int> g;

std::unique_ptr<int>& get_ref() {
    return g;
}

int main() {
    // get_ref() returns an lvalue reference, so copy is forbidden for move-only types.
    std::unique_ptr<int> p = get_ref();
    return 0;
}
