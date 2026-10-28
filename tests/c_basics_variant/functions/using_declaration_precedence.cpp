#include <cstdio>

namespace first {
int choose(int n) { return n + 30; }
}

namespace second {
int choose(int n) { return n + 40; }
}

int main() {
    using first::choose;
    {
        using namespace second;
        int selected = choose(2);
        std::printf("%d\n", selected);
    }
    return 0;
}
