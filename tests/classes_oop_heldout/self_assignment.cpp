#include <cstdio>

class Buffer {
public:
    int size;
    Buffer(int s) : size(s) {}
    Buffer& operator=(const Buffer& rhs) {
        if (this == &rhs) {
            std::printf("Buffer self-assign ignored for size=%d\n", size);
            return *this;
        }
        std::printf("Buffer assign %d -> %d\n", size, rhs.size);
        size = rhs.size;
        return *this;
    }
};

struct Point2D {
    int px;
    int py;
};

int main() {
    std::printf("custom self-assign test\n");
    Buffer b(512);
    b = b;
    std::printf("b.size = %d\n", b.size);

    std::printf("default self-assign test\n");
    Point2D pt = {15, 25};
    pt = pt;
    std::printf("pt = (%d, %d)\n", pt.px, pt.py);
    return 0;
}
