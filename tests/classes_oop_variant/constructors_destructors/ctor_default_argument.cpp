#include <cstdio>

struct Ticket {
    int id;
    int seats;
public:
    Ticket(int value, int booked = 4) : id(value), seats(booked) {
        std::printf("ticket %d %d\n", id, seats);
    }
    int total() const { return id + seats; }
};

int main() {
    Ticket a(10);
    std::printf("a=%d\n", a.total());
    Ticket b(10, 1);
    std::printf("b=%d\n", b.total());
    return 0;
}
