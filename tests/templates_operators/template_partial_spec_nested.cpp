#include <iostream>

template <typename T>
struct Traits {
    using Category = const char*;
};

template <typename T>
struct Traits<T*> {
    using Category = int;
    struct Node {
        int val;
    };
};

int main() {
    Traits<double*>::Category cat = 1;
    Traits<double*>::Node node{42};
    std::cout << cat << " " << node.val << std::endl;
    return 0;
}
