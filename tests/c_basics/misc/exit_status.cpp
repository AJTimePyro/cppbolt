#include <cstdio>
#include <cstdlib>

int main() {
    std::printf("before exit\n");
    std::exit(0);
    std::printf("after exit\n");
    return 0;
}
