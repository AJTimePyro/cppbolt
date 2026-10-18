#include <iostream>
#include <vector>

struct Tracked {
    int id;
    Tracked(int id_ = 0) : id(id_) {
        std::cout << "ctor " << id << std::endl;
    }
    Tracked(const Tracked& o) : id(o.id) {
        std::cout << "copy " << id << std::endl;
    }
    ~Tracked() {
        std::cout << "dtor " << id << std::endl;
    }
};

int main() {
    {
        std::vector<Tracked> vec;
        vec.push_back(Tracked(1));
    }
    std::cout << "done" << std::endl;
    return 0;
}
