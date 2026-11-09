#include <cstdio>

void report(int& x) { printf("lvalue ref: %d\n", x); }
void report(int&& x) { printf("rvalue ref: %d\n", x); }

template <typename T>
void forwarder(T&& arg) {
    report(static_cast<T&&>(arg));
}

int main() {
    int val = 100;
    forwarder(val);
    forwarder(200);
    return 0;
}
