#include <cstdio>

class Token {
public:
    int code;
    int extra;
    Token(int c, int e) : code(c), extra(e) {
        std::printf("Token ctor (%d, %d)\n", code, extra);
    }
    Token(const Token& other) : code(other.code + 5), extra(other.extra * 2) {
        std::printf("Token copy ctor (%d, %d)\n", code, extra);
    }
    Token& operator=(const Token& other) {
        std::printf("Token assign (%d,%d) -> (%d,%d)\n", code, extra, other.code, other.extra);
        code = other.code;
        extra = other.extra;
        return *this;
    }
};

int main() {
    Token t1(10, 2);
    Token t2 = t1;
    Token t3(99, 99);
    t3 = t2;
    std::printf("final codes: %d, %d, %d\n", t1.code, t2.code, t3.code);
    std::printf("final extras: %d, %d, %d\n", t1.extra, t2.extra, t3.extra);
    return 0;
}
