#include <cstdio>

int main() {
    unsigned int u = 4294967295u;
    std::printf("u_max: %u\n", u);
    u = u + 1u;
    std::printf("u_wrap: %u\n", u);
    u = u + 10u;
    std::printf("u_add: %u\n", u);
    u = u - 15u;
    std::printf("u_underflow: %u\n", u);
    return 0;
}
