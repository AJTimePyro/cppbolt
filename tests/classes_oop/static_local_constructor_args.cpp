#include <cstdio>

struct Stored {
    int value;
    Stored(int initial) : value(initial) {
        std::printf("ctor %d\n", value);
    }
    ~Stored() {
        std::printf("dtor %d\n", value);
    }
};

int read_static() {
    static Stored stored(42);
    return stored.value;
}

int main() {
    int first = read_static();
    int second = read_static();
    std::printf("%d %d\n", first, second);
    return 0;
}
