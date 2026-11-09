#include <cstdio>
#include <memory>
#include <utility>

struct Node {
    int val;
    Node(int v) : val(v) { printf("create Node: %d\n", val); }
    ~Node() { printf("destroy Node: %d\n", val); }
    void ping() const { printf("Node ping: %d\n", val); }
};

int main() {
    std::unique_ptr<Node> p1(new Node(42));
    if (p1) {
        p1->ping();
        printf("deref: %d\n", (*p1).val);
    }

    std::unique_ptr<Node> p2 = std::move(p1);
    printf("p1 is null: %d, p2 is non-null: %d\n", p1.get() == nullptr, p2.get() != nullptr);
    p2->ping();

    p2.reset(new Node(99));
    printf("-- exiting scope --\n");
    return 0;
}
