#include <cstdio>
#include <new>

struct Item {
    int id;
    Item(int v) : id(v) {}
    ~Item() { std::printf("~Item(%d)\n", id); }
};

typedef Item ItemAlias;

char buf[64];

int main() {
    ItemAlias* p = new (buf) ItemAlias(99);
    (*p).~ItemAlias();
    return 0;
}
