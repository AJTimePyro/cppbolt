#include <cstdio>

struct Config {
    int width = 800;
    int height = 600;
    int depth = 32;
};

int main() {
    Config c_default{};
    Config c_partial{1024};
    Config c_full{1920, 1080, 24};

    std::printf("c_default: %d %d %d\n", c_default.width, c_default.height, c_default.depth);
    std::printf("c_partial: %d %d %d\n", c_partial.width, c_partial.height, c_partial.depth);
    std::printf("c_full: %d %d %d\n", c_full.width, c_full.height, c_full.depth);

    int h = Config{1280}.height;
    std::printf("temp h: %d\n", h);

    Config* dyn = new Config{640};
    std::printf("dyn: %d %d %d\n", dyn->width, dyn->height, dyn->depth);
    delete dyn;

    return 0;
}
