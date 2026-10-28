#include <cstdio>

struct DisplayMode {
    int res_x;
    int res_y;
    int bpp;
    int refresh;
};

class RectBox {
    int left;
    int top;
    int right;
    int bottom;
public:
    RectBox(int l, int t, int r, int b) : left(l), top(t), right(r), bottom(b) {}
    void print_coords() {
        std::printf("RectBox: (%d,%d)-(%d,%d)\n", left, top, right, bottom);
    }
};

int main() {
    DisplayMode mode = {1920, 1080, 24, 60};
    std::printf("DisplayMode: %dx%d @ %dHz, %dbpp\n", mode.res_x, mode.res_y, mode.refresh, mode.bpp);

    RectBox box(5, 10, 50, 80);
    box.print_coords();
    return 0;
}
