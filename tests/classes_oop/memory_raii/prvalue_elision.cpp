#include <cstdio>

class Item {
public:
    int val;
    Item(int v) : val(v) {
        std::printf("Item ctor %d\n", val);
    }
    Item(const Item& other) : val(other.val + 100) {
        std::printf("Item copy ctor %d\n", val);
    }
    ~Item() {
        std::printf("Item dtor %d\n", val);
    }
};

Item create_item(int v) {
    return Item(v); // Prvalue return: copy ctor must NOT be called in C++17
}

void consume_item(Item it) { // By-value prvalue parameter: copy ctor must NOT be called in C++17
    std::printf("consume_item val: %d\n", it.val);
}

int main() {
    std::printf("--- test prvalue return ---\n");
    Item a = create_item(10);
    std::printf("a.val = %d\n", a.val);

    std::printf("--- test prvalue argument ---\n");
    consume_item(Item(20));

    std::printf("--- test end ---\n");
    return 0;
}
