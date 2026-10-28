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

void report(const char* label, const Transceiver& t) {
    std::printf("%s freq=%d, pwr=%d\n", label, t.ch.freq, t.power);
}

int main() {
    std::printf("create t1\n");
    {
        Transceiver t1(915, 25);
        std::printf("copy t2 = t1\n");
        Transceiver t2 = t1;
        report("t2", t2);

        std::printf("assign t3 = t2\n");
        Transceiver t3(433, 10);
        t3 = t2;
        report("t3", t3);
        std::printf("end of scope\n");
    }
    std::printf("finished\n");
    return 0;
}
