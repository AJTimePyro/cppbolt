#include <iostream>

int dtor_count = 0;

struct Item {
    int val;
    Item(int v) : val(v) {}
    ~Item() {
        dtor_count++;
    }
};

int test_break_and_return() {
    Item items[4] = {Item(1), Item(2), Item(3), Item(4)};
    dtor_count = 0;

    for (Item& item : items) {
        Item local(item.val * 10);
        if (item.val == 2) {
            continue;
        }
        if (item.val == 3) {
            break;
        }
    }
    std::cout << "locals destroyed: " << dtor_count << std::endl;
    return 0;
}

int main() {
    test_break_and_return();
    std::cout << "total destroyed: " << dtor_count << std::endl;
    return 0;
}
