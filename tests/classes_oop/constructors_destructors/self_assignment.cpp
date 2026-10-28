#include <cstdio>

class CustomAssign {
public:
    int val;
    CustomAssign(int v) : val(v) {}
    CustomAssign& operator=(const CustomAssign& other) {
        if (this == &other) {
            std::printf("Self-assignment detected for val=%d\n", val);
            return *this;
        }
        std::printf("Normal assignment %d -> %d\n", val, other.val);
        val = other.val;
        return *this;
    }
};

struct DefaultAssign {
    int x;
    int y;
};

int main() {
    std::printf("--- test CustomAssign self-assign ---\n");
    CustomAssign ca(42);
    ca = ca;
    std::printf("ca.val = %d\n", ca.val);

    std::printf("--- test DefaultAssign self-assign ---\n");
    DefaultAssign da = {10, 20};
    da = da;
    std::printf("da = (%d, %d)\n", da.x, da.y);
    return 0;
}
