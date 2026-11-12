#include <cstdio>
#include <algorithm>
#include <vector>

bool is_even(int x) { return x % 2 == 0; }
bool is_odd(int x) { return x % 2 != 0; }
bool is_gt_10(int x) { return x > 10; }
bool custom_eq(int a, int b) { return a == b; }
bool custom_less(int a, int b) { return a < b; }

void print_val(int x) {
    std::printf("%d ", x);
}

int main() {
    int arr[] = {2, 4, 6, 8, 10};
    int mix[] = {1, 2, 3, 4, 5, 6};

    // all_of, any_of, none_of
    std::printf("all_even: %d\n", std::all_of(arr, arr + 5, is_even));
    std::printf("all_even_mix: %d\n", std::all_of(mix, mix + 6, is_even));
    std::printf("any_even_mix: %d\n", std::any_of(mix, mix + 6, is_even));
    std::printf("none_gt_10: %d\n", std::none_of(arr, arr + 5, is_gt_10));

    // for_each, for_each_n
    std::printf("for_each: ");
    std::for_each(arr, arr + 3, print_val);
    std::printf("\nfor_each_n: ");
    std::for_each_n(arr, 4, print_val);
    std::printf("\n");

    // count, count_if
    int data[] = {1, 2, 2, 3, 2, 4, 5, 2};
    std::printf("count_2: %d\n", (int)std::count(data, data + 8, 2));
    std::printf("count_even: %d\n", (int)std::count_if(data, data + 8, is_even));

    // find, find_if, find_if_not
    auto it_f = std::find(data, data + 8, 3);
    std::printf("find_3: %d\n", (it_f != data + 8) ? *it_f : -1);
    auto it_fi = std::find_if(data, data + 8, is_gt_10);
    std::printf("find_gt_10: %d\n", it_fi == data + 8);
    auto it_fin = std::find_if_not(arr, arr + 5, is_even);
    std::printf("find_if_not_even: %d\n", it_fin == arr + 5);

    // adjacent_find
    int adj[] = {1, 2, 3, 3, 4, 5};
    auto it_adj = std::adjacent_find(adj, adj + 6);
    std::printf("adjacent_find: %d\n", (it_adj != adj + 6) ? *it_adj : -1);
    auto it_adj_custom = std::adjacent_find(adj, adj + 6, custom_eq);
    std::printf("adjacent_find_custom: %d\n", (it_adj_custom != adj + 6) ? *it_adj_custom : -1);

    // search & find_end
    int hay[] = {1, 2, 3, 4, 1, 2, 3, 4, 5};
    int needle[] = {2, 3, 4};
    auto it_search = std::search(hay, hay + 9, needle, needle + 3);
    std::printf("search_idx: %d\n", (int)(it_search - hay));
    auto it_find_end = std::find_end(hay, hay + 9, needle, needle + 3);
    std::printf("find_end_idx: %d\n", (int)(it_find_end - hay));

    // find_first_of
    int query[] = {7, 9, 3};
    auto it_ffo = std::find_first_of(hay, hay + 9, query, query + 3);
    std::printf("find_first_of: %d\n", (it_ffo != hay + 9) ? *it_ffo : -1);

    // search_n
    int repeated[] = {1, 2, 2, 2, 3, 4};
    auto it_sn = std::search_n(repeated, repeated + 6, 3, 2);
    std::printf("search_n_idx: %d\n", (int)(it_sn - repeated));
    auto it_sn4 = std::search_n(repeated, repeated + 6, 4, 2);
    std::printf("search_n_fail: %d\n", it_sn4 == repeated + 6);

    // mismatch 3-iter and 4-iter
    int a1[] = {1, 2, 3, 4, 5};
    int a2[] = {1, 2, 9, 4, 5};
    auto mm3 = std::mismatch(a1, a1 + 5, a2);
    std::printf("mismatch3: %d %d\n", *mm3.first, *mm3.second);
    auto mm4 = std::mismatch(a1, a1 + 5, a2, a2 + 5, custom_eq);
    std::printf("mismatch4: %d %d\n", *mm4.first, *mm4.second);

    // equal 3-iter and 4-iter
    int a3[] = {1, 2, 3};
    int a4[] = {1, 2, 3};
    int a5[] = {1, 2, 3, 4};
    std::printf("equal3: %d\n", std::equal(a3, a3 + 3, a4));
    std::printf("equal4_same: %d\n", std::equal(a3, a3 + 3, a4, a4 + 3));
    std::printf("equal4_diff_len: %d\n", std::equal(a3, a3 + 3, a5, a5 + 4));

    // lexicographical_compare
    std::printf("lex_comp_less: %d\n", std::lexicographical_compare(a3, a3 + 3, a5, a5 + 4));
    std::printf("lex_comp_custom: %d\n", std::lexicographical_compare(a5, a5 + 4, a3, a3 + 3, custom_less));

    return 0;
}
