#include <cstdio>

namespace N {
struct S {};
void free(S) { std::printf("N::free\n"); }
}

int main() {
    N::S s;
    free(s);
    return 0;
}
