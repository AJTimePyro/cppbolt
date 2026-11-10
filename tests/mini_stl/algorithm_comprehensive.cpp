#include <cstdio>
#include <algorithm>
#include <vector>

struct Item {
    int key;
    int val;
};

struct ItemKeyLess {
    bool operator()(const Item& a, const Item& b) const {
        return a.key < b.key;
    }
};

struct Greater {
    bool operator()(int a, int b) const {
        return a > b;
    }
};

int main() {
    // 1. Empty ranges
    int empty[1] = {0};
    int* ep = empty;
    std::printf("empty min_elem: %d\n", (int)(std::min_element(ep, ep) == ep));
    std::printf("empty max_elem: %d\n", (int)(std::max_element(ep, ep) == ep));
    std::printf("empty count: %d\n", (int)std::count(ep, ep, 42));
    std::printf("empty find: %d\n", (int)(std::find(ep, ep, 42) == ep));
    std::printf("empty is_sorted: %d\n", (int)std::is_sorted(ep, ep));
    std::printf("empty all_of: %d\n", (int)std::all_of(ep, ep, [](int x) { return x > 0; }));
    std::printf("empty any_of: %d\n", (int)std::any_of(ep, ep, [](int x) { return x > 0; }));
    std::printf("empty none_of: %d\n", (int)std::none_of(ep, ep, [](int x) { return x > 0; }));
    std::sort(ep, ep);
    std::reverse(ep, ep);

    // 2. Single element
    int single[1] = {42};
    std::printf("single min_elem: %d\n", *std::min_element(single, single + 1));
    std::printf("single is_sorted: %d\n", (int)std::is_sorted(single, single + 1));
    std::reverse(single, single + 1);
    std::printf("single after reverse: %d\n", single[0]);

    // 3. Custom functor and lambda
    int data[6] = {3, 1, 4, 1, 5, 9};
    std::sort(data, data + 6, Greater());
    std::printf("descending sort:");
    for (int i = 0; i < 6; ++i) std::printf(" %d", data[i]);
    std::printf("\n");

    int threshold = 3;
    int count_gt = std::count_if(data, data + 6, [threshold](int x) { return x > threshold; });
    std::printf("count > 3: %d\n", count_gt);

    // 4. Stability verification with stable_sort
    Item items[6] = {
        {2, 10},
        {1, 20},
        {2, 30},
        {1, 40},
        {2, 50},
        {1, 60}
    };
    std::stable_sort(items, items + 6, ItemKeyLess());
    std::printf("stable_sort preserved order:");
    for (int i = 0; i < 6; ++i) {
        std::printf(" (%d,%d)", items[i].key, items[i].val);
    }
    std::printf("\n");

    // 5. Clamp edge cases
    std::printf("clamp below: %d\n", std::clamp(5, 10, 20));
    std::printf("clamp inside: %d\n", std::clamp(15, 10, 20));
    std::printf("clamp above: %d\n", std::clamp(25, 10, 20));

    // 6. Lexicographical compare
    int p1[] = {1, 2, 3};
    int p2[] = {1, 2, 3, 4};
    int p3[] = {1, 2, 4};
    std::printf("lex_cmp shorter: %d\n", (int)std::lexicographical_compare(p1, p1 + 3, p2, p2 + 4));
    std::printf("lex_cmp longer: %d\n", (int)std::lexicographical_compare(p2, p2 + 4, p1, p1 + 3));
    std::printf("lex_cmp greater: %d\n", (int)std::lexicographical_compare(p1, p1 + 3, p3, p3 + 3));

    // 7. Unique with consecutive runs
    int u_data[] = {1, 1, 2, 3, 3, 3, 2, 2, 4};
    int* u_end = std::unique(u_data, u_data + 9);
    std::printf("unique:");
    for (int* it = u_data; it != u_end; ++it) {
        std::printf(" %d", *it);
    }
    std::printf("\n");

    return 0;
}
