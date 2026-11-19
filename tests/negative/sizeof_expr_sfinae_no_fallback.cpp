#include <cstdio>

template<class T>
auto f(T x) -> decltype(sizeof(x.value)) {
    return 0;
}

struct No {};

int main() {
    No n;
    return f(n);
}
