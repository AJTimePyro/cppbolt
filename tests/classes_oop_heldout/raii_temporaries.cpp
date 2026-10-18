#include <cstdio>

class TempBuffer {
public:
    int capacity;
    TempBuffer(int c) : capacity(c) {
        std::printf("alloc temp buf %d\n", capacity);
    }
    ~TempBuffer() {
        std::printf("free temp buf %d\n", capacity);
    }
};

int inspect_capacity(const TempBuffer& b) {
    return b.capacity;
}

int main() {
    std::printf("expr 1 start\n");
    int c1 = inspect_capacity(TempBuffer(128));
    std::printf("expr 1 result: %d\n", c1);

    std::printf("expr 2 start\n");
    int c2 = inspect_capacity(TempBuffer(256));
    std::printf("expr 2 result: %d\n", c2);

    std::printf("cond test\n");
    if (inspect_capacity(TempBuffer(512)) == 512) {
        std::printf("matched 512\n");
    }
    std::printf("cond done\n");

    std::printf("total cap: %d\n", c1 + c2);
    return 0;
}
