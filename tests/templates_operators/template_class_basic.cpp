#include <cstdio>

template <typename T>
class Holder {
    T value;
public:
    Holder(T val) : value(val) {}
    T get() const { return value; }
    void set(T val) { value = val; }
};

int main() {
    Holder<int> h1(42);
    std::printf("h1: %d\n", h1.get());
    h1.set(99);
    std::printf("h1 updated: %d\n", h1.get());

    Holder<char> h2('X');
    std::printf("h2: %c\n", h2.get());
    return 0;
}
