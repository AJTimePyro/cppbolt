#include <cstdio>
#include <tuple>
#include <utility>
#include <memory>

struct MoveTracker {
    int val;
    MoveTracker(int v) : val(v) {}
    MoveTracker(const MoveTracker& o) : val(o.val) { std::printf("copy\n"); }
    MoveTracker(MoveTracker&& o) : val(o.val) { std::printf("move\n"); }
};

int main() {
    std::tuple<MoveTracker> t(MoveTracker(100));
    std::printf("before move\n");
    MoveTracker m(std::get<0>(std::move(t)));
    std::printf("val: %d\n", m.val);

    std::tuple<std::unique_ptr<int>> ut(std::unique_ptr<int>(new int(42)));
    std::unique_ptr<int> p = std::get<0>(std::move(ut));
    std::printf("unique: %d\n", *p);

    return 0;
}
