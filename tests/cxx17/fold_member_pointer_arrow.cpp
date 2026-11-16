#include <cstdio>

struct Node3 {
    int value;
    int get_val() const {
        return value;
    }
};

struct Node2 {
    Node3* next;
};

struct Node1 {
    Node2* next;
};

template <typename Ptr, typename... Ptrs>
int traverse_arrow(Ptr p, Ptrs... ptrs) {
    return (p ->* ... ->* ptrs);
}

template <typename... Args>
int traverse_unary_arrow(Args... args) {
    return (... ->* args);
}

struct InnerMethod {
    int factor;
    int multiply(int x) {
        return x * factor;
    }
};

struct OuterMethod {
    InnerMethod* inner;
};

template <typename Ptr, typename... Ptrs>
int invoke_arrow(Ptr p, int arg, Ptrs... ptrs) {
    return (p ->* ... ->* ptrs)(arg);
}

int main() {
    Node3 n3;
    n3.value = 100;

    Node2 n2;
    n2.next = &n3;

    Node1 n1;
    n1.next = &n2;

    int val = traverse_arrow(&n1, &Node1::next, &Node2::next, &Node3::value);
    std::printf("traverse_arrow: %d\n", val);

    int val_unary = traverse_unary_arrow(&n1, &Node1::next, &Node2::next, &Node3::value);
    std::printf("traverse_unary_arrow: %d\n", val_unary);

    Node3* pn3 = &n3;
    int val_single = traverse_arrow(pn3, &Node3::value);
    std::printf("traverse_single: %d\n", val_single);

    InnerMethod im;
    im.factor = 7;
    OuterMethod om;
    om.inner = &im;

    int invoked = invoke_arrow(&om, 6, &OuterMethod::inner, &InnerMethod::multiply);
    std::printf("invoked_arrow: %d\n", invoked);

    return 0;
}
