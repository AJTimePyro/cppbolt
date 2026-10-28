#include <cstdio>

class Counter {
public:
    static int total_instances;
    int id;

    Counter() {
        total_instances++;
        id = total_instances;
    }
    ~Counter() {
        total_instances--;
    }
    static int get_total() {
        return total_instances;
    }
};

int Counter::total_instances = 0;

int main() {
    std::printf("initial: %d\n", Counter::get_total());
    Counter c1;
    std::printf("after c1: %d, id: %d\n", Counter::get_total(), c1.id);
    {
        Counter c2;
        Counter c3;
        std::printf("inside scope: %d\n", Counter::get_total());
    }
    std::printf("after scope: %d\n", Counter::get_total());
    return 0;
}
