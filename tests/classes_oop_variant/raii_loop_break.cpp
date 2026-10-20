#include <cstdio>

class IterGuard {
    int step;
public:
    IterGuard(int s) : step(s) {
        std::printf("enter step %d\n", step);
    }
    ~IterGuard() {
        std::printf("exit step %d\n", step);
    }
};

int main() {
    for (int k = 10; k < 15; ++k) {
        IterGuard g(k);
        if (k == 11) {
            std::printf("skip step %d\n", k);
            continue;
        }
        if (k == 13) {
            std::printf("abort at step %d\n", k);
            break;
        }
    }
    std::printf("loop finished\n");
    return 0;
}
