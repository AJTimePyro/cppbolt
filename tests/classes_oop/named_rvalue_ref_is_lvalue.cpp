#include <cstdio>
#include <utility>

void inspect(int&) {
    printf("lvalue ref\n");
}

void inspect(int&&) {
    printf("rvalue ref\n");
}

void forward_test(int&& r) {
    // In C++, r is a named variable, so expression `r` is an lvalue.
    inspect(r);
    // std::move(r) turns it into an xvalue (rvalue).
    inspect(std::move(r));
}

int main() {
    int x = 10;
    forward_test(std::move(x));
    return 0;
}
