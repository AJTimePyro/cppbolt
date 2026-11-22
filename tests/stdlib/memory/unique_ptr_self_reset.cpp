#include <cstdio>
#include <memory>

int main() {
    std::unique_ptr<int> p(new int(99));
    // Self-reset: resetting with its own pointer must not delete the pointer
    p.reset(p.get());
    int* dummy = new int(12345);
    (void)dummy;
    if (!p || *p != 99) {
        printf("FAILED: %d\n", p ? *p : -1);
        return 1;
    }
    printf("SUCCESS: %d\n", *p);
    delete dummy;
    return 0;
}
