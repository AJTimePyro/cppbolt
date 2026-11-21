#include <cstdio>

struct Box { using type = int; };

// A dependent nested type from a default template argument resolves in the
// signature even when it is not named by a parameter declaration.
template <typename T, typename U = typename T::type> int f(T, U* p) { return *p; }

int main() {
    int v = 9;
    std::printf("%d\n", f(Box{}, &v));
    return 0;
}
