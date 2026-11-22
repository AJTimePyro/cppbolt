#include <cstdio>
#include <vector>
#include <utility>

struct Item {
    int id;
    Item(int i) : id(i) {}
    Item(const Item& o) : id(o.id) { printf("item copy: %d\n", id); }
    Item(Item&& o) noexcept : id(o.id) { o.id = -1; printf("item move: %d\n", id); }
    ~Item() { if (id >= 0) printf("item dtor: %d\n", id); }
};

int main() {
    std::vector<Item> v;
    v.push_back(Item{1});
    v.push_back(Item{2});

    printf("-- moving vector --\n");
    std::vector<Item> v2 = std::move(v);
    printf("v size: %d, v2 size: %d\n", (int)v.size(), (int)v2.size());
    printf("-- exiting main --\n");
    return 0;
}
