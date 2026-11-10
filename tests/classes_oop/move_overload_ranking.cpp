#include <cstdio>

struct Probe { int val; };

void process(const Probe& p) {
    printf("copy: %d\n", p.val);
}

void process(Probe&& p) {
    printf("move: %d\n", p.val);
}

int main() {
    Probe p{10};
    process(p);            // lvalue -> copy
    process(Probe{20});    // rvalue -> move
    return 0;
}
