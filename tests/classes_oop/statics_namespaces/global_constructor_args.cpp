#include <cstdio>

class Item {
public:
    int id;
    Item(int n) : id(n) {
        std::printf("Item %d\n", id);
    }
    ~Item() {
        std::printf("~Item %d\n", id);
    }
};

class Pair {
public:
    int x;
    int y;
    Pair(int a, int b) : x(a), y(b) {
        std::printf("Pair %d %d\n", x, y);
    }
    ~Pair() {
        std::printf("~Pair %d %d\n", x, y);
    }
};

Item global_item(1);
Item g_first(10), g_second(20);
Pair global_pair(3, 4);

int main() {
    Item local_item(2);
    std::printf("main global_item: %d\n", global_item.id);
    std::printf("main local_item: %d\n", local_item.id);
    std::printf("main g_first: %d, g_second: %d\n", g_first.id, g_second.id);
    std::printf("main global_pair: %d %d\n", global_pair.x, global_pair.y);
    return 0;
}
