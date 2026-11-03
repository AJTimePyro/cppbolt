#include <cstdio>

class Tree {
public:
    struct Node {
        int key;
        int height;

        Node(int k, int h) : key(k), height(h) {}
    };

    int sum_node(const Node& n) const {
        return n.key + n.height;
    }
};

int main() {
    Tree::Node n(15, 3);
    Tree t;
    std::printf("Node key: %d, height: %d\n", n.key, n.height);
    std::printf("Tree sum: %d\n", t.sum_node(n));
    return 0;
}
