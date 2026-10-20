#include <cstdio>

int main() {
    short s = -1234;
    unsigned short us = 65535;
    printf("int=%hd %hu\n", s, us);
    printf("exp=%E %e\n", 1234.5, 1234.5);
    printf("zero=%08.2f %010.4f\n", 3.14159, -3.14159);
    printf("flags=%+f % f %+08.2f % 08.2f\n", 1.5, 1.5, 3.14159, 3.14159);
    printf("str=[%10s] [%-10s] [%.3s] [%10.3s] [%8.2s]\n", "hi", "hi", "hello", "hello", "hello");
    return 0;
}
