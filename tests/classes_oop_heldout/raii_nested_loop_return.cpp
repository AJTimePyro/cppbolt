#include <cstdio>

class ScopeProbe {
public:
    int level;
    ScopeProbe(int l) : level(l) {
        std::printf("Probe OPEN %d\n", level);
    }
    ~ScopeProbe() {
        std::printf("Probe CLOSE %d\n", level);
    }
};

int search_grid() {
    ScopeProbe s1(1);
    ScopeProbe s2(2);

    for (int r = 0; r < 4; ++r) {
        ScopeProbe row(100 + r);
        for (int c = 0; c < 3; ++c) {
            ScopeProbe cell(200 + c);
            if (r == 1 && c == 2) {
                std::printf("found target at (%d,%d)\n", r, c);
                return 888;
            }
        }
    }
    return 0;
}

int main() {
    std::printf("start search\n");
    int val = search_grid();
    std::printf("end search: %d\n", val);
    return 0;
}
