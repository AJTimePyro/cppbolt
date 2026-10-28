#include <cstdio>

class Resource {
public:
    int id;
    Resource(int id_) : id(id_) {
        std::printf("Resource ctor %d\n", id);
    }
    ~Resource() {
        std::printf("Resource dtor %d\n", id);
    }
};

class Box {
public:
    Resource res;
    int tag;
    Box(int r, int t) : res(r), tag(t) {
        std::printf("Box ctor (%d, %d)\n", res.id, tag);
    }
};

int main() {
    std::printf("--- create b1 ---\n");
    {
        Box b1(10, 100);
        std::printf("--- copy b2 = b1 ---\n");
        Box b2 = b1;
        std::printf("b2: res=%d, tag=%d\n", b2.res.id, b2.tag);

        std::printf("--- assign b3 = b2 ---\n");
        Box b3(20, 200);
        b3 = b2;
        std::printf("b3: res=%d, tag=%d\n", b3.res.id, b3.tag);
        std::printf("--- exiting scope ---\n");
    }
    std::printf("--- finished ---\n");
    return 0;
}
