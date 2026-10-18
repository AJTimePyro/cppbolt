#include <cstdio>
#include <vector>

int main() {
    std::vector<int> v;
    v.push_back(42);
    for (int i = 0; i < 50; ++i) {
        v.push_back(v[0]);
    }
    printf("push_back aliasing passed: size=%d, val=%d\n", (int)v.size(), v[50]);

    std::vector<int> v2;
    v2.emplace_back(99);
    for (int i = 0; i < 50; ++i) {
        v2.emplace_back(v2[0]);
    }
    printf("emplace_back aliasing passed: size=%d, val=%d\n", (int)v2.size(), v2[50]);
    return 0;
}
