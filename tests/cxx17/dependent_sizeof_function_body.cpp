#include <cstdio>

struct Big { int a, b, c; };
struct H { typedef Big type; };
struct Wide { double d; };
struct HWide { typedef Wide type; };

template <class T>
int size_in_body() {
    char data[sizeof(typename T::type)];
    return (int)sizeof(data);
}

template <class T>
int align_in_body() {
    return (int)alignof(typename T::type);
}

int main() {
    printf("%d %d\n", size_in_body<H>(), align_in_body<HWide>());
    return 0;
}
