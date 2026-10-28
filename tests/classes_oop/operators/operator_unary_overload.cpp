#include <cstdio>

class Counter {
    int count;
public:
    Counter(int c) : count(c) {}
    int get() const { return count; }
};

int operator-(const Counter& c) { return -c.get(); }
bool operator!(const Counter& c) { return !c.get(); }
int operator~(const Counter& c) { return ~c.get(); }
int operator&(const Counter& c) { return c.get() & 0xff; }
int operator*(const Counter& c, int k) { return c.get() * k; }
int operator+(const Counter& c, int k) { return c.get() + k; }

int main() {
    Counter c(21);
    std::printf("neg=%d\n", -c);
    std::printf("not=%d\n", !c);
    std::printf("inv=%d\n", ~c);
    std::printf("and=%d\n", &c);
    std::printf("mul=%d\n", c * 3);
    std::printf("add=%d\n", c + 4);
    return 0;
}
