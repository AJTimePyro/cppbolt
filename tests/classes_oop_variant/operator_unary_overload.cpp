#include <cstdio>

struct Amount {
    int cents;
public:
    Amount(int c) : cents(c) {}
    int raw() const { return cents; }
};

int operator-(const Amount& a) { return -a.raw(); }
bool operator!(const Amount& a) { return !a.raw(); }
int operator~(const Amount& a) { return ~a.raw(); }
int operator&(const Amount& a) { return a.raw() & 0x0f; }
int operator*(const Amount& a, int k) { return a.raw() * k; }
int operator+(const Amount& a, int k) { return a.raw() + k; }

int main() {
    Amount a(9);
    std::printf("neg=%d\n", -a);
    std::printf("not=%d\n", !a);
    std::printf("inv=%d\n", ~a);
    std::printf("and=%d\n", &a);
    std::printf("mul=%d\n", a * 5);
    std::printf("add=%d\n", a + 6);
    return 0;
}
