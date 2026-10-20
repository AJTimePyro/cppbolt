#include <cstdio>

class Guard {
public:
    int id;
    Guard(int i) : id(i) {
        std::printf("enter Guard %d\n", id);
    }
    ~Guard() {
        std::printf("leave Guard %d\n", id);
    }
};

bool evaluate(Guard g, bool outcome) {
    std::printf("evaluate %d -> %d\n", g.id, outcome ? 1 : 0);
    return outcome;
}

int main() {
    std::printf("--- check AND shortcut ---\n");
    if (evaluate(Guard(11), false) && evaluate(Guard(22), true)) {
        std::printf("AND taken\n");
    }
    std::printf("--- after AND ---\n");

    std::printf("--- check OR shortcut ---\n");
    if (evaluate(Guard(33), true) || evaluate(Guard(44), false)) {
        std::printf("OR taken\n");
    }
    std::printf("--- after OR ---\n");

    std::printf("--- check Ternary cond true ---\n");
    int a = 1 ? evaluate(Guard(55), true) : evaluate(Guard(66), false);
    std::printf("a = %d\n", a);

    std::printf("--- check Ternary cond false ---\n");
    int b = 0 ? evaluate(Guard(77), true) : evaluate(Guard(88), false);
    std::printf("b = %d\n", b);

    return 0;
}
