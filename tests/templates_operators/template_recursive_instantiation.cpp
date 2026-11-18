#include <cstdio>

// The in-progress specialization must satisfy the recursive Node<T>* reference
// without starting a second instantiation.
template <typename T> struct Node {
    T value;
    Node<T>* next;
    Node(T v) : value(v), next(nullptr) {}
};

int main() {
    Node<int> a(5), b(9);
    a.next = &b;
    std::printf("%d %d\n", a.value, a.next->value);
    return 0;
}
