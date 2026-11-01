struct Item {
    ~Item() {}
};

struct Other {};
typedef Other OtherAlias;

int main() {
    Item* p = nullptr;
    p->~OtherAlias();
    return 0;
}
