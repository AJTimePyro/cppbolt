#include <cstdio>

template <typename T>
struct Box {
    T val;
};

template <typename T>
T add_default(T a, T b = 100) {
    return a + b;
}

template <typename T>
int extract(T item) {
    return item.val;
}

template <typename T>
T deref_ptr(T* const p) {
    return *p;
}

int main() {
    printf("%d\n", add_default(5));
    printf("%d\n", add_default(5, 20));

    Box<int> b;
    b.val = 77;
    printf("%d\n", extract<Box<int>>(b));

    int val = 999;
    int* const ptr = &val;
    printf("%d\n", deref_ptr(ptr));
    return 0;
}
