#include <cstdio>

// The variant checks the same initialization forms with a different class
// shape and a different order.

struct Holder {
    int value;
    Holder(int start) : value(start) {
        std::printf("Holder %d\n", start);
    }
    Holder(const Holder& other) : value(other.value) {
        std::printf("Holder copy %d\n", other.value);
    }
};

Holder build() {
    return Holder(21);
}

int main() {
    Holder source(5);
    Holder copied(source);
    std::printf("copied %d\n", copied.value);

    Holder built(build());
    std::printf("built %d\n", built.value);

    Holder assigned = build();
    std::printf("assigned %d\n", assigned.value);

    Holder wrapped(Holder(9));
    std::printf("wrapped %d\n", wrapped.value);

    Holder same = Holder(4);
    std::printf("same %d\n", same.value);
    return 0;
}
