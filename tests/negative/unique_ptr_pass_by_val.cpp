#include <memory>

void sink(std::unique_ptr<int> p) {
    (void)p;
}

int main() {
    std::unique_ptr<int> p1(new int(10));
    sink(p1);
    return 0;
}
