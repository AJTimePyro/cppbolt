#include <cstdio>

class TrackTemp {
public:
    int id;
    TrackTemp(int i) : id(i) {
        std::printf("+ TrackTemp %d\n", id);
    }
    ~TrackTemp() {
        std::printf("- TrackTemp %d\n", id);
    }
};

bool check_val(TrackTemp t, bool ret) {
    std::printf("check_val %d returning %d\n", t.id, ret ? 1 : 0);
    return ret;
}

int main() {
    std::printf("--- test AND false left ---\n");
    if (check_val(TrackTemp(1), false) && check_val(TrackTemp(2), true)) {
        std::printf("AND branch 1 taken\n");
    }
    std::printf("--- after AND 1 ---\n");

    std::printf("--- test OR true left ---\n");
    if (check_val(TrackTemp(3), true) || check_val(TrackTemp(4), false)) {
        std::printf("OR branch 1 taken\n");
    }
    std::printf("--- after OR 1 ---\n");

    std::printf("--- test Ternary true ---\n");
    int r1 = 1 ? check_val(TrackTemp(5), true) : check_val(TrackTemp(6), false);
    std::printf("r1=%d\n", r1);

    std::printf("--- test Ternary false ---\n");
    int r2 = 0 ? check_val(TrackTemp(7), true) : check_val(TrackTemp(8), false);
    std::printf("r2=%d\n", r2);

    return 0;
}
