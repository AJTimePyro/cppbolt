#include <cstdio>
#include <utility>

int global_val = 42;

int& get_lvalue_ref() {
    return global_val;
}

void inspect_type(int&) {
    printf("lvalue reference\n");
}

void inspect_type(int&&) {
    printf("rvalue reference\n");
}

template <typename T>
void forward_check(T&& arg) {
    inspect_type(std::forward<T>(arg));
}

int main() {
    forward_check(get_lvalue_ref());
    return 0;
}
