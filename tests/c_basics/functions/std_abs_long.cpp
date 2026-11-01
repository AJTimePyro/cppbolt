#include <cmath>
#include <cstdio>

int main() {
    long a = -123456L;
    long long b = -9876543210LL;
    std::printf("%ld\n", std::abs(a));
    std::printf("%lld\n", std::abs(b));
    return 0;
}
