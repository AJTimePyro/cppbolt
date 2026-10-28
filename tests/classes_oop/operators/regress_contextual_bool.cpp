#include <cstdio>

struct Status {
    int code;
    Status(int c) : code(c) {}
    explicit operator bool() const {
        return code == 0;
    }
};

int main() {
    Status s_ok(0);
    Status s_err(5);

    if (s_ok) {
        std::printf("s_ok is true in if\n");
    }
    if (s_err) {
        std::printf("BUG: s_err is false but treated as true\n");
    }
    if (!s_err) {
        std::printf("!s_err is true in if\n");
    }
    std::printf("ternary: %d\n", s_ok ? 1 : 2);
    std::printf("ternary err: %d\n", s_err ? 1 : 2);
    int count = 0;
    while (s_ok && count < 2) {
        std::printf("while loop iter %d\n", count);
        count++;
    }
    return 0;
}
