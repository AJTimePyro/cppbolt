#include <cstdio>

template<class T>
int f(T x, decltype(x) y) {
    return (int)(x + y);
}

template<class T>
int g(T x, decltype(x) y, decltype(y) z) {
    return (int)(x + y + z);
}

template<class T>
decltype(T()) h(T x) {
    return x;
}

int main() {
    printf("%d %d %d\n", f(1, 2), g(1, 2, 3), h(5));
    return 0;
}
