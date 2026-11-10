#include <cstdio>
#include <vector>
#include <utility>

struct Tracked {
    int val;
    Tracked(int v = 0) : val(v) {}
    Tracked(const Tracked& o) : val(o.val) { printf("copy: %d\n", val); }
    Tracked(Tracked&& o) noexcept : val(o.val) { o.val = 0; printf("move: %d\n", val); }
    Tracked& operator=(Tracked&& o) noexcept {
        val = o.val;
        o.val = 0;
        printf("move assign: %d\n", val);
        return *this;
    }
    ~Tracked() {}
};

int main() {
    std::vector<Tracked> v1;
    v1.push_back(Tracked{10});
    v1.push_back(Tracked{20});

    std::vector<Tracked> v2;
    v2 = std::move(v1);
    printf("v1 size: %d, v2 size: %d\n", (int)v1.size(), (int)v2.size());
    for (size_t i = 0; i < v2.size(); ++i) {
        printf("v2[%d] = %d\n", (int)i, v2[i].val);
    }
    return 0;
}
