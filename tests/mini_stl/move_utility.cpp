#include <cstdio>
#include <utility>

struct Widget {
    int val;
    Widget(int v = 0) : val(v) {}
    Widget(const Widget& o) : val(o.val) { printf("copy widget\n"); }
    Widget(Widget&& o) : val(o.val) { o.val = 0; printf("move widget\n"); }
};

void consume(Widget w) {
    printf("consumed: %d\n", w.val);
}

template <typename T>
void relay(T&& arg) {
    consume(std::forward<T>(arg));
}

int main() {
    Widget w{55};
    consume(std::move(w));
    printf("w after move: %d\n", w.val);

    relay(Widget{77});

    std::pair<Widget, int> p1(Widget{10}, 20);
    std::pair<Widget, int> p2 = std::move(p1);
    printf("p2 val: %d, p1 val: %d\n", p2.first.val, p1.first.val);
    return 0;
}
