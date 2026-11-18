#include <cstdio>

// Distinct template arguments must keep distinct cached specializations.
template <typename T> struct Slot { static int code() { return sizeof(T); } };

int main() {
    std::printf("%d %d %d\n", Slot<char>::code(), Slot<int>::code(), Slot<double>::code());
    return 0;
}
