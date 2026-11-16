#include <cstdio>

struct FakeStream {
    FakeStream& operator<<(int x) {
        printf("%d ", x);
        return *this;
    }
    FakeStream& operator<<(const char* s) {
        printf("%s ", s);
        return *this;
    }
};

template <typename Stream, typename... Args>
Stream& print_all(Stream& s, Args... args) {
    return (s << ... << args);
}

int main() {
    FakeStream stream;
    print_all(stream, 1, "hello", 42, "world");
    printf("\n");
    return 0;
}
