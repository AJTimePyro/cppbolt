#include <cmath>
#include <cstdio>

int main() {
    long a = 123456L;
    long z1 = 0L;
    long long b = 9876543210LL;
    long long z2 = 0LL;
    std::printf("%ld\n", std::abs(a));
    std::printf("%ld\n", std::abs(z1));
    std::printf("%lld\n", std::abs(b));
    std::printf("%lld\n", std::abs(z2));
    return 0;
}
