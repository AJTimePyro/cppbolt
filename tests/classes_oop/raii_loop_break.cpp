#include <cstdio>

class ScopeObj {
    int id;
public:
    ScopeObj(int i) : id(i) {
        std::printf("enter %d\n", id);
    }
    ~ScopeObj() {
        std::printf("exit %d\n", id);
    }
};

int main() {
    for (int i = 0; i < 4; ++i) {
        ScopeObj loop_obj(i);
        if (i == 1) {
            std::printf("continuing at %d\n", i);
            continue; // should destroy loop_obj(1)
        }
        if (i == 2) {
            std::printf("breaking at %d\n", i);
            break; // should destroy loop_obj(2)
        }
    }
    std::printf("done loop\n");
    return 0;
}
