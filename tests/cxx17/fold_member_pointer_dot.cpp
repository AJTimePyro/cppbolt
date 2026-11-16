#include <cstdio>

struct Level3 {
    int value;
    int get_double() const {
        return value * 2;
    }
};

struct Level2 {
    Level3 l3;
};

struct Level1 {
    Level2 l2;
};

template <typename Root, typename... Ptrs>
int extract(Root& root, Ptrs... pms) {
    return (root .* ... .* pms);
}

template <typename... Args>
int extract_unary_left(Args... args) {
    return (... .* args);
}

template <typename Root, typename... Ptrs>
int invoke(Root& root, int arg, Ptrs... pms) {
    return (root .* ... .* pms)(arg);
}

struct InnerMethod {
    int factor;
    int multiply(int x) {
        return x * factor;
    }
};

struct OuterMethod {
    InnerMethod inner;
};

int main() {
    Level1 l1;
    l1.l2.l3.value = 42;

    int val = extract(l1, &Level1::l2, &Level2::l3, &Level3::value);
    std::printf("extract: %d\n", val);

    int val_unary = extract_unary_left(l1, &Level1::l2, &Level2::l3, &Level3::value);
    std::printf("extract_unary_left: %d\n", val_unary);

    Level3 l3;
    l3.value = 10;
    int val_single = extract(l3, &Level3::value);
    std::printf("extract_single: %d\n", val_single);

    OuterMethod om;
    om.inner.factor = 5;
    int invoked = invoke(om, 10, &OuterMethod::inner, &InnerMethod::multiply);
    std::printf("invoked: %d\n", invoked);

    return 0;
}
