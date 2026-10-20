#include <cstdio>

class Channel {
public:
    int freq;
    Channel(int f) : freq(f) {
        std::printf("Channel ctor %d\n", freq);
    }
    ~Channel() {
        std::printf("Channel dtor %d\n", freq);
    }
};

class Transceiver {
public:
    Channel ch;
    int power;
    Transceiver(int f, int p) : ch(f), power(p) {
        std::printf("Transceiver init (%d, %d)\n", ch.freq, power);
    }
};

int main() {
    std::printf("create t1\n");
    {
        Transceiver t1(915, 25);
        std::printf("copy t2 = t1\n");
        Transceiver t2 = t1;
        std::printf("t2 freq=%d, pwr=%d\n", t2.ch.freq, t2.power);

        std::printf("assign t3 = t2\n");
        Transceiver t3(433, 10);
        t3 = t2;
        std::printf("t3 freq=%d, pwr=%d\n", t3.ch.freq, t3.power);
        std::printf("end of scope\n");
    }
    std::printf("finished\n");
    return 0;
}
