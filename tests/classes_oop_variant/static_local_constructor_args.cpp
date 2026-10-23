#include <cstdio>

struct Cached {
    int payload;
    Cached(int start) : payload(start) {
        std::printf("made %d\n", payload);
    }
    ~Cached() {
        std::printf("gone %d\n", payload);
    }
};

int cached_value() {
    static Cached cached(73);
    return cached.payload;
}

void show_cached() {
    int one = cached_value();
    int two = cached_value();
    std::printf("%d %d\n", one, two);
}

int main() {
    show_cached();
    return 0;
}
