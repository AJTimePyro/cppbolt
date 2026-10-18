#include <cstdio>
#include <cstdlib>

int main() {
    const int N = 10000000;
    char* is_prime = (char*)malloc(N);
    for (int i = 0; i < N; ++i) is_prime[i] = 1;
    is_prime[0] = 0;
    is_prime[1] = 0;
    for (int p = 2; p * p < N; ++p) {
        if (is_prime[p]) {
            for (int i = p * p; i < N; i += p) {
                is_prime[i] = 0;
            }
        }
    }
    int count = 0;
    for (int i = 0; i < N; ++i) {
        if (is_prime[i]) count++;
    }
    free(is_prime);
    printf("Primes up to %d: %d\n", N, count);
    return 0;
}
