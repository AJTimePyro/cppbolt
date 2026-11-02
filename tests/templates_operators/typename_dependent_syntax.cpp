#include <cstdio>

template <typename T>
struct Wrapper {
    typedef typename T::value_type value_type;

    value_type get(typename T::value_type* p) {
        return *p;
    }
};

struct IntHolder {
    typedef int value_type;
};

int main() {
    Wrapper<IntHolder> w;
    IntHolder::value_type v = 42;
    std::printf("Got: %d\n", w.get(&v));
    return 0;
}
