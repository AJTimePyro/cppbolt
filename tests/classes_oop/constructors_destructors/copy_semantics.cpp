#include <cstdio>

class Item {
public:
    int val;
    Item(int v) : val(v) {
        std::printf("Item ctor %d\n", val);
    }
    Item(const Item& other) : val(other.val + 10) {
        std::printf("Item copy ctor %d\n", val);
    }
    Item& operator=(const Item& other) {
        std::printf("Item assign %d -> %d\n", val, other.val);
        val = other.val;
        return *this;
    }
};

int main() {
    Item a(5);
    Item b = a; // copy ctor
    Item c(1);
    c = b; // assignment
    std::printf("final values: %d, %d, %d\n", a.val, b.val, c.val);
    return 0;
}
