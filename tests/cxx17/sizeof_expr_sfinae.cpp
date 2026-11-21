#include <cstdio>

struct Has { int value; };
struct No {};

// sizeof(x.value) is an unevaluated operand, but it is still type-checked.
// An invalid member rejects the template so the concrete overload wins.
template<class T>
auto size_of_value(T x) -> decltype(sizeof(x.value)) { return 1; }
int size_of_value(No) { return 2; }

int main() {
    Has h{1};
    No n;
    std::printf("size_of_value(Has)=%d size_of_value(No)=%d\n",
                size_of_value(h), size_of_value(n));
    int arr[4];
    std::printf("sizeof(expr)=%zu sizeof(type)=%zu alignof(type)=%zu\n",
                sizeof(arr[0]), sizeof(int), alignof(int));
    return 0;
}
