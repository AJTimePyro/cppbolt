struct Foo {
    explicit Foo(int x) {}
};

Foo make_foo() {
    return {10};
}

int main() {
    return 0;
}
