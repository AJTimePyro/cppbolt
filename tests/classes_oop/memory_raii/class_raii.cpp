#include <cstdio>

class Resource {
    int id;
public:
    Resource(int id_) : id(id_) {
        std::printf("Construct %d\n", id);
    }
    ~Resource() {
        std::printf("Destruct %d\n", id);
    }
    void use() {
        std::printf("Use %d\n", id);
    }
};

void test() {
    Resource r2(2);
    r2.use();
}

int main() {
    std::printf("Start main\n");
    {
        Resource r1(1);
        r1.use();
        test();
    }
    std::printf("End main\n");
    return 0;
}
