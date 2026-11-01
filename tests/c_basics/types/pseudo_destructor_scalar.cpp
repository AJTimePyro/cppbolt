#include <cstdio>

typedef int T;

int g_calls = 0;
int* get_ptr(int* p) {
    g_calls++;
    return p;
}

int main() {
    T val = 123;
    T* p = &val;
    p->~T();
    val.~T();
    get_ptr(p)->~T();
    std::printf("val=%d calls=%d\n", val, g_calls);
    return 0;
}
