#include <cstdio>

#define ID(x) x
#define INC(x) ((x)+1)

int main() {
    std::printf("%d\n", ID(INC)(4));
    return 0;
}
