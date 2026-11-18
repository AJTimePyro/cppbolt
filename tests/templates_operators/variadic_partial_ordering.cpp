#include <cstdio>

// Case 1: single before variadic
template <typename T>
void identify(T val) {
    printf("single: %d\n", (int)val);
}

template <typename T, typename... Rest>
void identify(T val, Rest... rest) {
    printf("variadic: %d\n", (int)val);
}

// Case 2: variadic before single (declaration order must not matter!)
template <typename T, typename... Rest>
void pick(T val, Rest... rest) {
    printf("pick variadic: %d\n", (int)val);
}

template <typename T>
void pick(T val) {
    printf("pick single: %d\n", (int)val);
}

int main() {
    identify(42);       // Must resolve to identify(T) by partial ordering
    identify(42, 100);  // Must resolve to identify(T, Rest...)
    pick(7);            // Must resolve to pick(T) even though variadic was declared first!
    pick(7, 88);        // Must resolve to pick(T, Rest...)
    return 0;
}
