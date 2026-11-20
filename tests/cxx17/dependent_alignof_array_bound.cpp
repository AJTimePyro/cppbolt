#include <cstdio>

struct HasChar { typedef char type; };
struct HasInt { typedef int type; };

template <class T>
struct Holder {
    char data[alignof(typename T::type)];
};

int main() {
    Holder<HasChar> a;
    Holder<HasInt> b;
    printf("%d %d\n", (int)sizeof(a.data), (int)sizeof(b.data));
    return 0;
}
