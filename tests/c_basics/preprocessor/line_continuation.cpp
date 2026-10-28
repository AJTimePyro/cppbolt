#include <cstdio>

#define ADD(x) \
    ((x) + 1)

int main() {
    std::printf("%d\n", ADD(4));
    std::printf("%s\n", "abc\
def");
    return 0;
}
