#include <cstdio>

int main() {
    long long a = 9000000000000000000LL;
    long long b = 123456789012345LL;
    long long sum = a + b;
    long long diff = a - b;
    long long div = a / 1000000000LL;

    std::printf("sum high: %lld\n", sum / 1000000000000LL);
    std::printf("diff high: %lld\n", diff / 1000000000000LL);
    std::printf("div: %lld\n", div);
    return 0;
}
