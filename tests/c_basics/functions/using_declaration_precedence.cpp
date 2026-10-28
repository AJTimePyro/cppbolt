#include <cstdio>

namespace alpha {
int pick(int value) { return value + 10; }
}

namespace beta {
int pick(int value) { return value + 20; }
}

int main() {
    using alpha::pick;
    {
        using namespace beta;
        std::printf("%d\n", pick(1));
    }
    return 0;
}
