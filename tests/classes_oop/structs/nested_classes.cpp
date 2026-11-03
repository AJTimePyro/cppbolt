#include <cstdio>

struct Outer {
    struct Inner {
        int val;
        int get_val() {
            return val;
        }
    };

    Inner make_inner(int v) {
        Inner i;
        i.val = v;
        return i;
    }
};

int main() {
    Outer o;
    Outer::Inner in = o.make_inner(42);
    std::printf("Inner val: %d\n", in.get_val());

    Outer::Inner direct;
    direct.val = 100;
    std::printf("Direct inner: %d\n", direct.val);
    return 0;
}
