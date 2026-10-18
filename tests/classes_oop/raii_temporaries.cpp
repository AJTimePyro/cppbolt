#include <cstdio>

class TempVal {
public:
    int val;
    TempVal(int v) : val(v) {
        std::printf("create temp %d\n", val);
    }
    ~TempVal() {
        std::printf("destroy temp %d\n", val);
    }
};

int get_val(const TempVal& a) {
    return a.val;
}

int main() {
    std::printf("before expr 1\n");
    int a = get_val(TempVal(10));
    std::printf("after expr 1: %d\n", a);

    std::printf("before expr 2\n");
    int b = get_val(TempVal(20));
    std::printf("after expr 2: %d\n", b);

    std::printf("before if\n");
    if (get_val(TempVal(30)) > 25) {
        std::printf("inside if branch\n");
    }
    std::printf("after if\n");

    std::printf("sum: %d\n", a + b);
    return 0;
}
