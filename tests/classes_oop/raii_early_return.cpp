#include <cstdio>

class Logger {
    const char* name;
public:
    Logger(const char* n) : name(n) {
        std::printf("+ %s\n", name);
    }
    ~Logger() {
        std::printf("- %s\n", name);
    }
};

int early_exit_test(int val) {
    Logger outer("outer");
    if (val < 0) {
        Logger inner("inner_early");
        return -1; // RAII must destroy inner_early then outer!
    }
    Logger normal("normal");
    return val * 2;
}

int main() {
    std::printf("Test 1:\n");
    int r1 = early_exit_test(-5);
    std::printf("r1: %d\n", r1);

    std::printf("Test 2:\n");
    int r2 = early_exit_test(10);
    std::printf("r2: %d\n", r2);
    return 0;
}
