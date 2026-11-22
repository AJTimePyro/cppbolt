#include <cstdio>

struct Item {
    int id;
};

struct Dispatcher {
    void handle(Item& it) {
        printf("mutable lvalue: %d\n", it.id);
    }
    void handle(const Item& it) {
        printf("const lvalue: %d\n", it.id);
    }
    void handle(Item&& it) {
        printf("rvalue: %d\n", it.id);
    }
};

int main() {
    Dispatcher d;
    Item a{1};
    const Item b{2};
    d.handle(a);
    d.handle(b);
    d.handle(Item{3});
    return 0;
}
