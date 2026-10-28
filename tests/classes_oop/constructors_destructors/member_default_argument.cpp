#include <cstdio>

class Greeter {
    int base;
public:
    Greeter(int b) : base(b) {}
    int scaled(int factor, int offset = 3) const {
        return base * factor + offset;
    }
    void report(int tag, int width = 2) const {
        std::printf("report %d %d\n", tag, width);
    }
};

int main() {
    Greeter g(4);
    std::printf("scaled=%d\n", g.scaled(5));
    g.report(1);
    g.report(2, 9);
    return 0;
}
