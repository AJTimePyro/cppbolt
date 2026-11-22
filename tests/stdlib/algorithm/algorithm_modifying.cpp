#include <cstdio>
#include <algorithm>
#include <vector>

int square(int x) { return x * x; }
int add(int a, int b) { return a + b; }
bool is_even(int x) { return x % 2 == 0; }
int next_ten() {
    static int val = 10;
    int res = val;
    val += 10;
    return res;
}

int main() {
    // copy, copy_n, copy_if, copy_backward
    int src[5] = {1, 2, 3, 4, 5};
    int dst[5] = {0};
    std::copy(src, src + 5, dst);
    std::printf("copy:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", dst[i]);
    std::printf("\n");

    int dst_n[5] = {0};
    std::copy_n(src, 3, dst_n);
    std::printf("copy_n:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", dst_n[i]);
    std::printf("\n");

    int dst_if[5] = {0};
    auto it_if = std::copy_if(src, src + 5, dst_if, is_even);
    std::printf("copy_if count=%d:", (int)(it_if - dst_if));
    for (int* p = dst_if; p != it_if; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int dst_bw[5] = {0};
    std::copy_backward(src, src + 3, dst_bw + 5);
    std::printf("copy_backward:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", dst_bw[i]);
    std::printf("\n");

    // move, move_backward
    int m_src[3] = {10, 20, 30};
    int m_dst[3] = {0};
    std::move(m_src, m_src + 3, m_dst);
    std::printf("move:");
    for (int i = 0; i < 3; ++i) std::printf(" %d", m_dst[i]);
    std::printf("\n");

    int mb_dst[5] = {0};
    std::move_backward(m_dst, m_dst + 3, mb_dst + 5);
    std::printf("move_backward:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", mb_dst[i]);
    std::printf("\n");

    // fill, fill_n
    int f_arr[5] = {0};
    std::fill(f_arr, f_arr + 3, 42);
    std::fill_n(f_arr + 3, 2, 99);
    std::printf("fill:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", f_arr[i]);
    std::printf("\n");

    // transform unary & binary
    int t_in[4] = {1, 2, 3, 4};
    int t_out[4] = {0};
    std::transform(t_in, t_in + 4, t_out, square);
    std::printf("transform_unary:");
    for (int i = 0; i < 4; ++i) std::printf(" %d", t_out[i]);
    std::printf("\n");

    int t_bin[4] = {0};
    std::transform(t_in, t_in + 4, t_out, t_bin, add);
    std::printf("transform_binary:");
    for (int i = 0; i < 4; ++i) std::printf(" %d", t_bin[i]);
    std::printf("\n");

    // generate, generate_n
    int g_arr[5] = {0};
    std::generate(g_arr, g_arr + 3, next_ten);
    std::generate_n(g_arr + 3, 2, next_ten);
    std::printf("generate:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", g_arr[i]);
    std::printf("\n");

    // remove, remove_if
    int r_arr[7] = {1, 2, 3, 2, 4, 2, 5};
    auto r_end = std::remove(r_arr, r_arr + 7, 2);
    std::printf("remove:");
    for (int* p = r_arr; p != r_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int rif_arr[6] = {1, 2, 3, 4, 5, 6};
    auto rif_end = std::remove_if(rif_arr, rif_arr + 6, is_even);
    std::printf("remove_if:");
    for (int* p = rif_arr; p != rif_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    // remove_copy, remove_copy_if
    int rc_src[5] = {1, 2, 3, 2, 4};
    int rc_dst[5] = {0};
    auto rc_end = std::remove_copy(rc_src, rc_src + 5, rc_dst, 2);
    std::printf("remove_copy:");
    for (int* p = rc_dst; p != rc_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int rcif_dst[5] = {0};
    auto rcif_end = std::remove_copy_if(rc_src, rc_src + 5, rcif_dst, is_even);
    std::printf("remove_copy_if:");
    for (int* p = rcif_dst; p != rcif_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    // replace, replace_if, replace_copy, replace_copy_if
    int rep_arr[5] = {1, 2, 3, 2, 5};
    std::replace(rep_arr, rep_arr + 5, 2, 20);
    std::printf("replace:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", rep_arr[i]);
    std::printf("\n");

    std::replace_if(rep_arr, rep_arr + 5, is_even, 0);
    std::printf("replace_if:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", rep_arr[i]);
    std::printf("\n");

    int rep_src[4] = {1, 2, 3, 4};
    int rep_out[4] = {0};
    std::replace_copy(rep_src, rep_src + 4, rep_out, 2, 99);
    std::printf("replace_copy:");
    for (int i = 0; i < 4; ++i) std::printf(" %d", rep_out[i]);
    std::printf("\n");

    std::replace_copy_if(rep_src, rep_src + 4, rep_out, is_even, 88);
    std::printf("replace_copy_if:");
    for (int i = 0; i < 4; ++i) std::printf(" %d", rep_out[i]);
    std::printf("\n");

    // swap_ranges
    int s1[3] = {1, 2, 3};
    int s2[3] = {7, 8, 9};
    std::swap_ranges(s1, s1 + 3, s2);
    std::printf("swap_ranges: s1=%d %d %d, s2=%d %d %d\n", s1[0], s1[1], s1[2], s2[0], s2[1], s2[2]);

    // reverse, reverse_copy
    int rev_arr[5] = {1, 2, 3, 4, 5};
    std::reverse(rev_arr, rev_arr + 5);
    std::printf("reverse:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", rev_arr[i]);
    std::printf("\n");

    int rev_dst[5] = {0};
    std::reverse_copy(rev_arr, rev_arr + 5, rev_dst);
    std::printf("reverse_copy:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", rev_dst[i]);
    std::printf("\n");

    // rotate, rotate_copy
    int rot_arr[5] = {1, 2, 3, 4, 5};
    auto rot_it = std::rotate(rot_arr, rot_arr + 2, rot_arr + 5);
    std::printf("rotate ret_val=%d:", *rot_it);
    for (int i = 0; i < 5; ++i) std::printf(" %d", rot_arr[i]);
    std::printf("\n");

    int rot_dst[5] = {0};
    std::rotate_copy(rot_arr, rot_arr + 3, rot_arr + 5, rot_dst);
    std::printf("rotate_copy:");
    for (int i = 0; i < 5; ++i) std::printf(" %d", rot_dst[i]);
    std::printf("\n");

    // unique, unique_copy
    int u_arr[8] = {1, 1, 2, 3, 3, 3, 4, 4};
    auto u_end = std::unique(u_arr, u_arr + 8);
    std::printf("unique:");
    for (int* p = u_arr; p != u_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    int uc_src[6] = {1, 2, 2, 3, 3, 4};
    int uc_dst[6] = {0};
    auto uc_end = std::unique_copy(uc_src, uc_src + 6, uc_dst);
    std::printf("unique_copy:");
    for (int* p = uc_dst; p != uc_end; ++p) std::printf(" %d", *p);
    std::printf("\n");

    return 0;
}
