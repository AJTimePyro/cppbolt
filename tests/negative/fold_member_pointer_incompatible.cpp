struct Foo {
    int x;
};

struct Bar {
    int y;
};

template <typename Root, typename... Ptrs>
int extract(Root& root, Ptrs... pms) {
    return (root .* ... .* pms);
}

int main() {
    Foo foo;
    extract(foo, &Bar::y);
    return 0;
}
