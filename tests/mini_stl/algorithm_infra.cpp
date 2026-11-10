#include <cstdio>
#include <iterator>
#include <algorithm>

int main() {
    int arr[] = {10, 20, 30, 40};
    int* p = arr;
    std::advance(p, 2);
    std::printf("*p: %d\n", *p);
    std::printf("dist: %ld\n", (long)std::distance(arr, p));
    std::iter_swap(arr, arr + 3);
    std::printf("swap: %d %d\n", arr[0], arr[3]);
    auto next_p = std::next(arr, 1);
    std::printf("next: %d\n", *next_p);
    auto prev_p = std::prev(arr + 3, 1);
    std::printf("prev: %d\n", *prev_p);
    return 0;
}
