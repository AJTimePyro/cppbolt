#include <cstdio>

template <typename T>
struct Holder {
    T value;
};

int inspect_ptr(Holder<int>* p) {
    return p->value;
}

int inspect_const_ref(const Holder<int>& r) {
    return r.value;
}

int main() {
    Holder<int> h;
    h.value = 42;
    printf("%d\n", inspect_ptr(&h));
    printf("%d\n", inspect_const_ref(h));
    return 0;
}
