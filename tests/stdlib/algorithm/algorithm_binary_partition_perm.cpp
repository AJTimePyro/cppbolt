#include <cstdio>
#include <algorithm>
#include <vector>

bool is_even(int x) { return x % 2 == 0; }
bool is_lt_5(int x) { return x < 5; }

int main() {
    // is_sorted, is_sorted_until
    int sorted_arr[] = {1, 2, 4, 4, 7, 9};
    int unsorted_arr[] = {1, 2, 5, 3, 7};
    std::printf("is_sorted_true: %d\n", std::is_sorted(sorted_arr, sorted_arr + 6));
    std::printf("is_sorted_false: %d\n", std::is_sorted(unsorted_arr, unsorted_arr + 5));
    auto it_until = std::is_sorted_until(unsorted_arr, unsorted_arr + 5);
    std::printf("is_sorted_until: %d\n", *it_until);

    // lower_bound, upper_bound, binary_search, equal_range
    auto lb = std::lower_bound(sorted_arr, sorted_arr + 6, 4);
    auto ub = std::upper_bound(sorted_arr, sorted_arr + 6, 4);
    std::printf("lower_bound_idx: %d\n", (int)(lb - sorted_arr));
    std::printf("upper_bound_idx: %d\n", (int)(ub - sorted_arr));
    std::printf("binary_search_has: %d\n", std::binary_search(sorted_arr, sorted_arr + 6, 7));
    std::printf("binary_search_miss: %d\n", std::binary_search(sorted_arr, sorted_arr + 6, 5));

    auto er = std::equal_range(sorted_arr, sorted_arr + 6, 4);
    std::printf("equal_range count=%d: [%d, %d)\n",
        (int)(er.second - er.first), (int)(er.first - sorted_arr), (int)(er.second - sorted_arr));

    // is_partitioned, partition, partition_point
    int part_data[] = {1, 3, 2, 4, 5, 6, 7};
    auto part_it = std::partition(part_data, part_data + 7, is_even);
    std::printf("is_partitioned: %d\n", std::is_partitioned(part_data, part_data + 7, is_even));
    std::printf("partition even count: %d\n", (int)(part_it - part_data));
    auto pp = std::partition_point(part_data, part_data + 7, is_even);
    std::printf("partition_point matches: %d\n", pp == part_it);

    // partition_copy
    int pc_src[] = {1, 2, 3, 4, 5, 6};
    int evens[6] = {0};
    int odds[6] = {0};
    auto pc_res = std::partition_copy(pc_src, pc_src + 6, evens, odds, is_even);
    std::printf("partition_copy evens:");
    for (int* p = evens; p != pc_res.first; ++p) std::printf(" %d", *p);
    std::printf("\npartition_copy odds:");
    for (int* p = odds; p != pc_res.second; ++p) std::printf(" %d", *p);
    std::printf("\n");

    // stable_partition
    int sp_data[] = {1, 2, 3, 4, 5, 6, 7, 8};
    auto sp_it = std::stable_partition(sp_data, sp_data + 8, is_even);
    std::printf("stable_partition evens:");
    for (int* p = sp_data; p != sp_it; ++p) std::printf(" %d", *p);
    std::printf("\nstable_partition odds:");
    for (int* p = sp_it; p != sp_data + 8; ++p) std::printf(" %d", *p);
    std::printf("\n");

    // is_permutation
    int p1[] = {1, 2, 3, 4, 5};
    int p2[] = {3, 1, 5, 2, 4};
    int p3[] = {1, 2, 3, 4, 6};
    std::printf("is_perm_true: %d\n", std::is_permutation(p1, p1 + 5, p2));
    std::printf("is_perm_false: %d\n", std::is_permutation(p1, p1 + 5, p3));
    std::printf("is_perm_4iter: %d\n", std::is_permutation(p1, p1 + 5, p2, p2 + 5));

    // next_permutation & prev_permutation
    int perm[] = {1, 2, 3};
    std::printf("permutations:\n");
    do {
        std::printf("  %d %d %d\n", perm[0], perm[1], perm[2]);
    } while (std::next_permutation(perm, perm + 3));

    int perm_rev[] = {3, 2, 1};
    std::printf("prev_permutations:\n");
    do {
        std::printf("  %d %d %d\n", perm_rev[0], perm_rev[1], perm_rev[2]);
    } while (std::prev_permutation(perm_rev, perm_rev + 3));

    return 0;
}
