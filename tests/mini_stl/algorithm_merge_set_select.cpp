#include <cstdio>
#include <algorithm>
#include <vector>

struct SimpleURBG {
    typedef unsigned int result_type;
    unsigned int state;
    SimpleURBG(unsigned int s = 12345) : state(s) {}
#ifdef __clang__
    static constexpr unsigned int min() { return 0; }
    static constexpr unsigned int max() { return 4294967295U; }
#else
    static unsigned int min() { return 0; }
    static unsigned int max() { return 4294967295U; }
#endif
    unsigned int operator()() {
        state = state * 1103515245U + 12345U;
        return state;
    }
};

int main() {
    // merge
    int m1[] = {1, 3, 5, 7};
    int m2[] = {2, 4, 6, 8};
    int m_out[8] = {0};
    std::merge(m1, m1 + 4, m2, m2 + 4, m_out);
    std::printf("merge:");
    for (int i = 0; i < 8; ++i) std::printf(" %d", m_out[i]);
    std::printf("\n");

    // inplace_merge
    int ip_arr[] = {1, 3, 5, 2, 4, 6};
    std::inplace_merge(ip_arr, ip_arr + 3, ip_arr + 6);
    std::printf("inplace_merge:");
    for (int i = 0; i < 6; ++i) std::printf(" %d", ip_arr[i]);
    std::printf("\n");

    // includes, set_union, set_intersection, set_difference, set_symmetric_difference
    int s1[] = {1, 2, 3, 4, 5};
    int s2[] = {2, 4};
    int s3[] = {3, 4, 5, 6, 7};
    std::printf("includes_true: %d\n", std::includes(s1, s1 + 5, s2, s2 + 2));
    std::printf("includes_false: %d\n", std::includes(s2, s2 + 2, s1, s1 + 5));

    int u_out[10] = {0};
    auto u_end = std::set_union(s1, s1 + 5, s3, s3 + 5, u_out);
    std::printf("set_union:");
    for (int* p = u_out; p != u_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int i_out[10] = {0};
    auto i_end = std::set_intersection(s1, s1 + 5, s3, s3 + 5, i_out);
    std::printf("set_intersection:");
    for (int* p = i_out; p != i_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int d_out[10] = {0};
    auto d_end = std::set_difference(s1, s1 + 5, s3, s3 + 5, d_out);
    std::printf("set_difference:");
    for (int* p = d_out; p != d_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int sd_out[10] = {0};
    auto sd_end = std::set_symmetric_difference(s1, s1 + 5, s3, s3 + 5, sd_out);
    std::printf("set_symmetric_difference:");
    for (int* p = sd_out; p != sd_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    // partial_sort, partial_sort_copy
    int ps_arr[] = {9, 7, 5, 3, 1, 8, 6, 4, 2};
    std::partial_sort(ps_arr, ps_arr + 4, ps_arr + 9);
    std::printf("partial_sort:");
    for (int i = 0; i < 4; ++i) std::printf(" %d", ps_arr[i]);
    std::printf("\n");

    int psc_src[] = {9, 2, 7, 1, 5};
    int psc_dst[3] = {0};
    std::partial_sort_copy(psc_src, psc_src + 5, psc_dst, psc_dst + 3);
    std::printf("partial_sort_copy:");
    for (int i = 0; i < 3; ++i) std::printf(" %d", psc_dst[i]);
    std::printf("\n");

    // nth_element
    int nth_arr[] = {7, 2, 1, 6, 8, 5, 3, 4};
    std::nth_element(nth_arr, nth_arr + 3, nth_arr + 8);
    std::printf("nth_element_at_3: %d\n", nth_arr[3]);

    // stable_sort
    int st_arr[] = {5, 2, 8, 1, 9, 3, 7, 4, 6};
    std::stable_sort(st_arr, st_arr + 9);
    std::printf("stable_sort:");
    for (int i = 0; i < 9; ++i) std::printf(" %d", st_arr[i]);
    std::printf("\n");

    // shuffle & sample
    SimpleURBG urbg1(42);
    int sh_arr[] = {1, 2, 3, 4, 5, 6, 7, 8};
    int orig_sh[] = {1, 2, 3, 4, 5, 6, 7, 8};
    std::shuffle(sh_arr, sh_arr + 8, urbg1);
    bool sh_perm = std::is_permutation(sh_arr, sh_arr + 8, orig_sh);
    bool sh_diff = false;
    for (int i = 0; i < 8; ++i) {
        if (sh_arr[i] != orig_sh[i]) sh_diff = true;
    }
    std::printf("shuffle_is_permutation: %d\n", (int)sh_perm);
    std::printf("shuffle_changed_order: %d\n", (int)sh_diff);

    SimpleURBG urbg2(99);
    int smp_src[] = {10, 20, 30, 40, 50, 60, 70, 80};
    int smp_dst[4] = {0};
    int* smp_end = std::sample(smp_src, smp_src + 8, smp_dst, 4, urbg2);
    std::printf("sample_count: %d\n", (int)(smp_end - smp_dst));
    bool smp_valid = true;
    for (int i = 0; i < 4; ++i) {
        bool found = false;
        for (int j = 0; j < 8; ++j) {
            if (smp_dst[i] == smp_src[j]) found = true;
        }
        if (!found) smp_valid = false;
    }
    std::printf("sample_all_elements_valid: %d\n", (int)smp_valid);

    return 0;
}
