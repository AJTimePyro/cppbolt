#include <cstdio>

class Buffer {
public:
    int slots[3];
    Buffer() {
        slots[0] = 4;
        slots[1] = 5;
        slots[2] = 6;
    }
    int& slot(int index) { return slots[index]; }
};

int main() {
    Buffer buffer;
    buffer.slot(0) = 40;
    buffer.slot(1) = 41;
    int& chosen = buffer.slot(0);
    std::printf("%d %d %d\n", buffer.slot(0), buffer.slot(1), chosen);
    return 0;
}
