#include <cstdio>
struct HasInt { typedef int type; };
struct NoType {};

template <class T> int f(T, char (*)[sizeof(typename T::type)]) { return 1; }
template <class T> int f(T, double) { return -1; }

int main() {
    HasInt h;
    char a[sizeof(int)];
    NoType n;
    printf("%d %d\n", f(h, &a), f(n, 0));
    return 0;
}
