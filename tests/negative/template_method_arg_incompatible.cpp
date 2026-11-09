struct Foo {
    template <typename T>
    void bar(T x, const char* s) {
        (void)x;
        (void)s;
    }
};

int main() {
    Foo f;
    // 123 cannot convert to const char*, so member template resolution must fail.
    f.bar(1, 123);
    return 0;
}
