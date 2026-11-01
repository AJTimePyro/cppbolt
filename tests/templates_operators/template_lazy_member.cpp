#include <cstdio>

template <typename T>
class LazyBox {
    T val;
public:
    LazyBox(T v) : val(v) {}
    T get() const { return val; }
    void bad_method() {
        val.nonexistent();
    }
};

int main() {
    LazyBox<int> b(123);
    std::printf("lazy get: %d\n", b.get());
    return 0;
}
