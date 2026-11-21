#include <cstdio>

struct Good { using type = int; };
struct Bad {};

// The template is discarded for Bad, so the non-template overload is selected.
template <typename T> int pick(T, typename T::type* = nullptr) { return 1; }
int pick(Bad) { return 2; }

int main() {
    std::printf("%d %d\n", pick(Good{}), pick(Bad{}));
    return 0;
}
