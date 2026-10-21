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
        std::printf("%d\n", choose(2));
    }
    return 0;
}
