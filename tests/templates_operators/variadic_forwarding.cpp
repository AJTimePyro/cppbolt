#include <cstdio>

void report(int& x) {
    printf("lval:%d ", x);
}

void report(int&& x) {
    printf("rval:%d ", x);
}

void sink() {
    printf("\n");
}

template <typename Head, typename... Tail>
void sink(Head&& head, Tail&&... tail) {
    report(static_cast<Head&&>(head));
    sink(static_cast<Tail&&>(tail)...);
}

void target(int& x, int&& y) {
    report(static_cast<int&>(x));
    report(static_cast<int&&>(y));
    printf("\n");
}

template <typename... Args>
void relay(Args&&... args) {
    target(static_cast<Args&&>(args)...);
}

int main() {
    int a = 1;
    int b = 2;
    sink(a, 10, b, 20);
    relay(a, 30);
    return 0;
}
