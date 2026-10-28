#include <cstdio>

int main() {
    unsigned int max_u32 = 4294967295U;
    unsigned int wrapped = max_u32 + 1U;
    std::printf("wrapped: %u\n", wrapped);

    unsigned int zero = 0U;
    unsigned int underflow = zero - 1U;
    std::printf("underflow: %u\n", underflow);

    unsigned short us = 65535;
    us = us + 1; // standard integer promotion then assigned
    std::printf("us wrap: %u\n", (unsigned int)us);
    return 0;
}
