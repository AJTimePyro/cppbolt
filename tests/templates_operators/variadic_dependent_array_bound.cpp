#include <stdio.h>

template <typename... Ts>
struct ArrayHolder {
    int direct[sizeof...(Ts)];
    int offset[sizeof...(Ts) + 2];

    void init() {
        direct[0] = sizeof...(Ts);
        offset[0] = sizeof...(Ts) + 2;
    }
};

int main() {
    ArrayHolder<int, double, char> h;
    h.init();
    printf("direct: %d, offset: %d\n", h.direct[0], h.offset[0]);
    return 0;
}
