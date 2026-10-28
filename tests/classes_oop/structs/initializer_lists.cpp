#include <cstdio>

struct Config {
    int width;
    int height;
    int depth;
};

class Container {
    int a;
    int b;
public:
    Container(int x, int y) : a(x), b(y) {}
    void print() {
        std::printf("Container: %d, %d\n", a, b);
    }
};

int main() {
    Config cfg = {800, 600, 32};
    std::printf("Config: %d x %d x %d\n", cfg.width, cfg.height, cfg.depth);

    Container c(10, 20);
    c.print();
    return 0;
}
