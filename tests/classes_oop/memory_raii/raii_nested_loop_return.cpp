#include <cstdio>

class Tracer {
public:
    int id;
    Tracer(int id_) : id(id_) {
        std::printf("ctor %d\n", id);
    }
    ~Tracer() {
        std::printf("dtor %d\n", id);
    }
};

int find_and_return() {
    Tracer outer1(1);
    Tracer outer2(2);

    for (int i = 0; i < 5; ++i) {
        Tracer loop_obj(10 + i);
        if (i == 2) {
            Tracer inner_obj(100);
            std::printf("early return at i=%d\n", i);
            return 42;
        }
    }
    return 0;
}

int main() {
    std::printf("before call\n");
    int res = find_and_return();
    std::printf("after call result: %d\n", res);
    return 0;
}
