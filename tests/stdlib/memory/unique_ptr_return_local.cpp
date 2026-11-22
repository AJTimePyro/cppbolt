#include <cstdio>
#include <memory>

std::unique_ptr<int> create_ptr(int val) {
    std::unique_ptr<int> p(new int(val));
    return p; // C++11 implicit move on local automatic object
}

int main() {
    std::unique_ptr<int> res = create_ptr(42);
    if (!res || *res != 42) {
        printf("FAILED\n");
        return 1;
    }
    printf("SUCCESS: %d\n", *res);
    return 0;
}
