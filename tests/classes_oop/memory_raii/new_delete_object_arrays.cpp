#include <cstdio>

class Element {
public:
    static int count;
    int id;
    Element() {
        count++;
        id = count;
        std::printf("Element default ctor id=%d\n", id);
    }
    ~Element() {
        std::printf("Element dtor id=%d\n", id);
    }
};

int Element::count = 0;

int main() {
    std::printf("--- allocating array ---\n");
    Element* arr = new Element[3];
    std::printf("--- array created ---\n");
    std::printf("elem 0 id=%d\n", arr[0].id);
    std::printf("elem 1 id=%d\n", arr[1].id);
    std::printf("elem 2 id=%d\n", arr[2].id);
    std::printf("--- deleting array ---\n");
    delete[] arr;
    std::printf("--- deleted ---\n");
    return 0;
}
