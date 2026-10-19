#include <cstdio>

int g_large[1000000];
short g_short[1024];
char g_chars[8];

int main() {
    g_large[0] = 7;
    g_large[250000] = 250;
    g_large[999999] = 99;
    g_short[0] = -1;
    g_short[1023] = 1023;
    g_chars[0] = 'A';
    g_chars[7] = 'Z';
    printf("large=%d,%d,%d short=%d,%d chars=%c,%c\n",
           g_large[0], g_large[250000], g_large[999999],
           g_short[0], g_short[1023], g_chars[0], g_chars[7]);
    return 0;
}
