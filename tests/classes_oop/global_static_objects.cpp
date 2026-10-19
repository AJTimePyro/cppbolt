#include <cstdio>

class GlobalLogger {
public:
    int id;
    GlobalLogger() : id(1) {
        printf("GlobalLogger ctor\n");
    }
    ~GlobalLogger() {
        printf("GlobalLogger dtor\n");
    }
};

class SecondaryLogger {
public:
    int id;
    SecondaryLogger() : id(2) {
        printf("SecondaryLogger ctor\n");
    }
    ~SecondaryLogger() {
        printf("SecondaryLogger dtor\n");
    }
};

GlobalLogger g_log1;
SecondaryLogger g_log2;
int g_large_zero_array[10000];

int main() {
    printf("Entering main\n");
    printf("zero[0]=%d, zero[5000]=%d, zero[9999]=%d\n",
           g_large_zero_array[0], g_large_zero_array[5000], g_large_zero_array[9999]);
    g_large_zero_array[5000] = 777;
    printf("modified zero[5000]=%d\n", g_large_zero_array[5000]);
    printf("Exiting main\n");
    return 0;
}