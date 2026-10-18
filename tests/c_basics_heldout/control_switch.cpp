#include <cstdio>

int main() {
    int nums[5];
    nums[0] = 10;
    nums[1] = 20;
    nums[2] = 25;
    nums[3] = 30;
    nums[4] = 99;

    for (int i = 0; i < 5; i = i + 1) {
        int v = nums[i];
        switch (v) {
            case 10:
                std::printf("ten\n");
                break;
            case 20:
            case 25:
                std::printf("twenty_or_25: %d\n", v);
                break;
            case 30:
                std::printf("thirty\n");
                break;
            default:
                std::printf("other: %d\n", v);
                break;
        }
    }
    return 0;
}
