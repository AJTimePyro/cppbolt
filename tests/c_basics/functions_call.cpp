#include <cstdio>

int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

int fibonacci(int n) {
    if (n <= 0) return 0;
    if (n == 1) return 1;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    std::printf("fact(5) = %d\n", factorial(5));
    std::printf("fib(7) = %d\n", fibonacci(7));
    return 0;
}
