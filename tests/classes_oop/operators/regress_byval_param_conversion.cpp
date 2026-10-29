#include <cstdio>

struct Foo {
    int val;
    Foo(int v) : val(v) {}
    operator int() const {
        return val;
    }
};

struct WithCopyCtor {
    int val;
    WithCopyCtor(int v) : val(v) {}
    WithCopyCtor(const WithCopyCtor& other) : val(other.val + 100) {}
    operator int() const {
        return val;
    }
};

int read_foo(Foo f) {
    return f.val;
}

Foo pass_through_foo(Foo f) {
    return f;
}

int read_with_copy(WithCopyCtor w) {
    return w.val;
}

int main() {
    Foo f(42);
    int v1 = read_foo(f);
    Foo f2 = f;
    Foo f3(f);
    Foo f4 = pass_through_foo(f);
    std::printf("byval: %d, copy_init: %d, direct_init: %d, ret_byval: %d\n", v1, f2.val, f3.val, f4.val);

    Foo f_other(999);
    int v_other = read_foo(f_other);
    std::printf("byval_other: %d\n", v_other);

    WithCopyCtor w(10);
    int w_byval = read_with_copy(w);
    WithCopyCtor w2 = w;
    std::printf("custom_copy byval: %d, copy_init: %d\n", w_byval, w2.val);

    return 0;
}
