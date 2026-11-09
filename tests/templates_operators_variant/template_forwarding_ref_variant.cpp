#include <cstdio>

void check_val(double& d) { printf("lval double: %.1f\n", d); }
void check_val(double&& d) { printf("rval double: %.1f\n", d); }

template <typename T>
void relay(T&& x) {
    check_val(static_cast<T&&>(x));
}

int main() {
    double pi = 3.14;
    relay(pi);
    relay(2.71);
    return 0;
}
