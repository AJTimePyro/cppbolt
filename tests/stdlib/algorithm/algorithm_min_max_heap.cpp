#include <cstdio>
#include <vector>
#include <algorithm>

bool greater_cmp(int a, int b) {
    return a > b;
}

int main() {
    // clamp
    std::printf("clamp: %d %d %d\n",
        std::clamp(15, 10, 20),
        std::clamp(5, 10, 20),
        std::clamp(25, 10, 20));
    std::printf("clamp_comp: %d\n", std::clamp(5, 20, 10, greater_cmp));

    // min / max
    std::printf("min_val: %d, max_val: %d\n", std::min(10, 20), std::max(10, 20));
    std::printf("min_comp: %d, max_comp: %d\n",
        std::min(10, 20, greater_cmp), std::max(10, 20, greater_cmp));

    // minmax
    int x1 = 42, y1 = 17;
    auto mm = std::minmax(x1, y1);
    std::printf("minmax: %d %d\n", mm.first, mm.second);
    auto mm_comp = std::minmax(x1, y1, greater_cmp);
    std::printf("minmax_comp: %d %d\n", mm_comp.first, mm_comp.second);

    // min_element, max_element, minmax_element
    int arr[] = {3, 1, 4, 1, 5, 9, 2};
    auto min_it = std::min_element(arr, arr + 7);
    auto max_it = std::max_element(arr, arr + 7);
    auto mm_it = std::minmax_element(arr, arr + 7);
    std::printf("min_elem: %d, max_elem: %d\n", *min_it, *max_it);
    std::printf("minmax_elem: %d %d\n", *mm_it.first, *mm_it.second);

    // Heap operations
    std::vector<int> h;
    h.push_back(10);
    h.push_back(20);
    h.push_back(15);
    h.push_back(30);
    h.push_back(5);

    std::make_heap(h.begin(), h.end());
    std::printf("is_heap: %d\n", std::is_heap(h.begin(), h.end()));
    std::printf("heap_top: %d\n", h[0]);

    h.push_back(40);
    std::push_heap(h.begin(), h.end());
    std::printf("after_push top: %d\n", h[0]);

    std::pop_heap(h.begin(), h.end());
    int popped = h[h.size() - 1];
    h.pop_back();
    std::printf("popped: %d, new_top: %d\n", popped, h[0]);

    std::sort_heap(h.begin(), h.end());
    std::printf("sort_heap:");
    for (size_t i = 0; i < h.size(); ++i) {
        std::printf(" %d", h[i]);
    }
    std::printf("\n");

    return 0;
}
