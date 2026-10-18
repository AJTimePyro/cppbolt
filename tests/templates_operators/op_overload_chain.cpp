#include <cstdio>

class ChainNum {
public:
    int val;
    ChainNum(int v = 0) : val(v) {}

    ChainNum operator+(const ChainNum& o) const {
        return ChainNum(val + o.val);
    }
    ChainNum operator*(const ChainNum& o) const {
        return ChainNum(val * o.val);
    }
};

int main() {
    ChainNum n1(2);
    ChainNum n2(3);
    ChainNum n3(4);

    ChainNum res = n1 + n2 * n3; // precedence check: 2 + (3 * 4) = 14
    std::printf("chain res: %d\n", res.val);
    return 0;
}
