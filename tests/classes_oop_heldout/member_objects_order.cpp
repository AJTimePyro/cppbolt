#include <cstdio>

class SubUnit {
    int code;
public:
    SubUnit(int c) : code(c) {
        std::printf("SubUnit ctor %d\n", code);
    }
    ~SubUnit() {
        std::printf("SubUnit dtor %d\n", code);
    }
};

class MasterUnit {
    SubUnit s1;
    SubUnit s2;
public:
    MasterUnit(int a, int b) : s1(a), s2(b) {
        std::printf("MasterUnit active\n");
    }
    ~MasterUnit() {
        std::printf("MasterUnit shutdown\n");
    }
};

int main() {
    std::printf("system start\n");
    {
        MasterUnit mu(101, 202);
    }
    std::printf("system stop\n");
    return 0;
}
