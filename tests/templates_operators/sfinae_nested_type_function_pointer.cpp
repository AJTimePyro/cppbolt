#include <cstdio>

struct Bad {};
void handler(int) {}

// The missing T::type discards the function-pointer overload, leaving the
// void* overload as the only candidate.
template <typename T> int f(void (*p)(typename T::type)) { return 1; }
template <typename T> int f(void*) { return 2; }

int main() {
    std::printf("%d\n", f<Bad>(nullptr));
    return 0;
}
