#include <cstdio>

int g_large[1000000];

int main() {
    g_large[0] = 111;
    g_large[500000] = 555;
    g_large[999999] = 999;
    printf("g[0]=%d, g[500k]=%d, g[999k]=%d\n", g_large[0], g_large[500000], g_large[999999]);
    return 0;
}
