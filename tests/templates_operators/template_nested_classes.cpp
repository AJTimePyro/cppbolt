#include <iostream>

template <typename T>
class Container {
public:
    struct Node {
        T value;
        Node* next;
        Node(T val, Node* n = nullptr) : value(val), next(n) {}
    };

    typedef Node NodeType;
    typedef T ValueType;

private:
    Node* head;

public:
    Container() : head(nullptr) {}
    ~Container() {
        Node* curr = head;
        while (curr) {
            Node* next = curr->next;
            delete curr;
            curr = next;
        }
    }

    void push(T val) {
        head = new Node(val, head);
    }

    void print() const {
        Node* curr = head;
        while (curr) {
            std::cout << curr->value << " ";
            curr = curr->next;
        }
        std::cout << std::endl;
    }

    typename Container<T>::NodeType* get_head() const {
        return head;
    }
};

template <typename C>
void print_first(const C& c) {
    typename C::NodeType* node = c.get_head();
    if (node) {
        typename C::ValueType v = node->value;
        std::cout << "first: " << v << std::endl;
    }
}

int main() {
    Container<int> int_c;
    int_c.push(10);
    int_c.push(20);
    int_c.push(30);
    int_c.print();
    print_first(int_c);

    Container<double> dbl_c;
    dbl_c.push(1.5);
    dbl_c.push(2.5);
    dbl_c.print();
    print_first(dbl_c);

    typename Container<int>::Node direct_node(99, nullptr);
    std::cout << "direct node: " << direct_node.value << std::endl;

    return 0;
}
