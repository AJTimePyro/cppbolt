struct Foo {
    explicit Foo(int x) {}
};

void consume(Foo f) {}

int main() {
    consume({10});
    return 0;
}
