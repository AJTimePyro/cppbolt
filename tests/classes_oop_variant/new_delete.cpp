#include <cstdio>

class Packet {
public:
    int id;
    int len;
    Packet(int i, int l) : id(i), len(l) {
        std::printf("Packet ctor [%d, %d]\n", id, len);
    }
    ~Packet() {
        std::printf("Packet dtor [%d, %d]\n", id, len);
    }
};

int main() {
    int* val = new int(256);
    std::printf("*val = %d\n", *val);
    delete val;

    Packet* p = new Packet(7, 512);
    std::printf("p->len = %d\n", p->len);
    delete p;

    int* buffer = new int[4];
    buffer[0] = 11;
    buffer[1] = 22;
    buffer[2] = 33;
    buffer[3] = 44;
    std::printf("buffer sum = %d\n", buffer[0] + buffer[1] + buffer[2] + buffer[3]);
    delete[] buffer;
    return 0;
}
