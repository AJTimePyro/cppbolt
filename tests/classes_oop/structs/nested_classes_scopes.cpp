#include <cstdio>

struct Outer {
    struct Middle {
        using ValueType = int;
        struct Inner {
            ValueType val;
            Inner(ValueType v) : val(v) {}
        };
    };
};

int main() {
    Outer::Middle::Inner item(42);
    std::printf("%d\n", item.val);
    return 0;
}
