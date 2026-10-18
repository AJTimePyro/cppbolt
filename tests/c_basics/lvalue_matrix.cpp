#include <cstdio>

enum Color { Red = 0, Green = 1, Blue = 2 };
struct S { int id; };

char g_char_buf[16] = { 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 0 };
int g_int_buf[8] = {10, 20, 30, 40, 50, 60, 70, 80};
double g_double_buf[8] = {1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8};
S g_struct_buf[8] = {{1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}};

char g_var_char = 'A';
struct Holder_g_char { char m; };
Holder_g_char g_holder_char = { 'A' };
char g_arr_char[4] = { 'A', 'A', 'B', 'B' };
void take_val_g_char(char x) { (void)x; }
void take_ref_g_char(char& x) { (void)x; }
char return_g_char() { return 'B'; }

short g_var_short = 12;
struct Holder_g_short { short m; };
Holder_g_short g_holder_short = { 12 };
short g_arr_short[4] = { 12, 12, 34, 34 };
void take_val_g_short(short x) { (void)x; }
void take_ref_g_short(short& x) { (void)x; }
short return_g_short() { return 34; }

int g_var_int = 101;
struct Holder_g_int { int m; };
Holder_g_int g_holder_int = { 101 };
int g_arr_int[4] = { 101, 101, 202, 202 };
void take_val_g_int(int x) { (void)x; }
void take_ref_g_int(int& x) { (void)x; }
int return_g_int() { return 202; }

long long g_var_long_long = 1000000000LL;
struct Holder_g_long_long { long long m; };
Holder_g_long_long g_holder_long_long = { 1000000000LL };
long long g_arr_long_long[4] = { 1000000000LL, 1000000000LL, 2000000000LL, 2000000000LL };
void take_val_g_long_long(long long x) { (void)x; }
void take_ref_g_long_long(long long& x) { (void)x; }
long long return_g_long_long() { return 2000000000LL; }

float g_var_float = 1.5f;
struct Holder_g_float { float m; };
Holder_g_float g_holder_float = { 1.5f };
float g_arr_float[4] = { 1.5f, 1.5f, 4.5f, 4.5f };
void take_val_g_float(float x) { (void)x; }
void take_ref_g_float(float& x) { (void)x; }
float return_g_float() { return 4.5f; }

double g_var_double = 2.25;
struct Holder_g_double { double m; };
Holder_g_double g_holder_double = { 2.25 };
double g_arr_double[4] = { 2.25, 2.25, 6.75, 6.75 };
void take_val_g_double(double x) { (void)x; }
void take_ref_g_double(double& x) { (void)x; }
double return_g_double() { return 6.75; }

bool g_var_bool = true;
struct Holder_g_bool { bool m; };
Holder_g_bool g_holder_bool = { true };
bool g_arr_bool[4] = { true, true, false, false };
void take_val_g_bool(bool x) { (void)x; }
void take_ref_g_bool(bool& x) { (void)x; }
bool return_g_bool() { return false; }

char* g_var_char_ptr = g_char_buf + 0;
struct Holder_g_char_ptr { char* m; };
Holder_g_char_ptr g_holder_char_ptr = { g_char_buf + 0 };
char* g_arr_char_ptr[4] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 2, g_char_buf + 2 };
void take_val_g_char_ptr(char* x) { (void)x; }
void take_ref_g_char_ptr(char*& x) { (void)x; }
char* return_g_char_ptr() { return g_char_buf + 2; }

int* g_var_int_ptr = g_int_buf + 0;
struct Holder_g_int_ptr { int* m; };
Holder_g_int_ptr g_holder_int_ptr = { g_int_buf + 0 };
int* g_arr_int_ptr[4] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 2, g_int_buf + 2 };
void take_val_g_int_ptr(int* x) { (void)x; }
void take_ref_g_int_ptr(int*& x) { (void)x; }
int* return_g_int_ptr() { return g_int_buf + 2; }

double* g_var_double_ptr = g_double_buf + 0;
struct Holder_g_double_ptr { double* m; };
Holder_g_double_ptr g_holder_double_ptr = { g_double_buf + 0 };
double* g_arr_double_ptr[4] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 2, g_double_buf + 2 };
void take_val_g_double_ptr(double* x) { (void)x; }
void take_ref_g_double_ptr(double*& x) { (void)x; }
double* return_g_double_ptr() { return g_double_buf + 2; }

S* g_var_struct_ptr = g_struct_buf + 0;
struct Holder_g_struct_ptr { S* m; };
Holder_g_struct_ptr g_holder_struct_ptr = { g_struct_buf + 0 };
S* g_arr_struct_ptr[4] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 2, g_struct_buf + 2 };
void take_val_g_struct_ptr(S* x) { (void)x; }
void take_ref_g_struct_ptr(S*& x) { (void)x; }
S* return_g_struct_ptr() { return g_struct_buf + 2; }

Color g_var_enum_color = Green;
struct Holder_g_enum_color { Color m; };
Holder_g_enum_color g_holder_enum_color = { Green };
Color g_arr_enum_color[4] = { Green, Green, Blue, Blue };
void take_val_g_enum_color(Color x) { (void)x; }
void take_ref_g_enum_color(Color& x) { (void)x; }
Color return_g_enum_color() { return Blue; }

int total_checks = 0;
unsigned int rolling_hash = 2166136261u;
void check(bool condition, const char* msg) {
    total_checks++;
    rolling_hash = rolling_hash * 16777619u ^ (condition ? 1u : 0u);
    if (!condition) {
        std::printf("FAILED: %s\n", msg);
    }
}

int main() {
    // Case 1: char_local_assign
    {
        char lv = 'A';
        lv = 'B';
        check(lv == 'B', "char_local_assign");
        std::printf("CASE 1: char_local_assign value=%d\n", (int)(lv));
    }

    // Case 2: char_local_compound_assign
    {
        char lv = 'A';
        lv += 1;
        check(lv == ('A') + 1, "char_local_compound_assign");
        std::printf("CASE 2: char_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 3: char_local_inc_dec
    {
        char lv = 'A';
        ++lv;
        --lv;
        check(lv == 'A', "char_local_inc_dec");
        std::printf("CASE 3: char_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 4: char_local_read_after_write
    {
        char lv = 'A';
        lv = 'B';
        char read_val = lv;
        check(read_val == 'B', "char_local_read_after_write");
        std::printf("CASE 4: char_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 5: char_local_pass_by_val
    {
        char lv = 'A';
        take_val_g_char(lv);
        check(lv == 'A', "char_local_pass_by_val");
        std::printf("CASE 5: char_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 6: char_local_pass_by_ref
    {
        char lv = 'A';
        take_ref_g_char(lv);
        check(lv == 'A', "char_local_pass_by_ref");
        std::printf("CASE 6: char_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 7: char_local_return_by_val
    {
        char lv = 'A';
        lv = return_g_char();
        check(lv == 'B', "char_local_return_by_val");
        std::printf("CASE 7: char_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 8: char_global_assign
    {
        g_var_char = 'A';
        g_var_char = 'B';
        check(g_var_char == 'B', "char_global_assign");
        std::printf("CASE 8: char_global_assign value=%d\n", (int)(g_var_char));
    }

    // Case 9: char_global_compound_assign
    {
        g_var_char = 'A';
        g_var_char += 1;
        check(g_var_char == ('A') + 1, "char_global_compound_assign");
        std::printf("CASE 9: char_global_compound_assign value=%d\n", (int)(g_var_char));
    }

    // Case 10: char_global_inc_dec
    {
        g_var_char = 'A';
        ++g_var_char;
        --g_var_char;
        check(g_var_char == 'A', "char_global_inc_dec");
        std::printf("CASE 10: char_global_inc_dec value=%d\n", (int)(g_var_char));
    }

    // Case 11: char_global_read_after_write
    {
        g_var_char = 'A';
        g_var_char = 'B';
        char read_val = g_var_char;
        check(read_val == 'B', "char_global_read_after_write");
        std::printf("CASE 11: char_global_read_after_write value=%d\n", (int)(g_var_char));
    }

    // Case 12: char_global_pass_by_val
    {
        g_var_char = 'A';
        take_val_g_char(g_var_char);
        check(g_var_char == 'A', "char_global_pass_by_val");
        std::printf("CASE 12: char_global_pass_by_val value=%d\n", (int)(g_var_char));
    }

    // Case 13: char_global_pass_by_ref
    {
        g_var_char = 'A';
        take_ref_g_char(g_var_char);
        check(g_var_char == 'A', "char_global_pass_by_ref");
        std::printf("CASE 13: char_global_pass_by_ref value=%d\n", (int)(g_var_char));
    }

    // Case 14: char_global_return_by_val
    {
        g_var_char = 'A';
        g_var_char = return_g_char();
        check(g_var_char == 'B', "char_global_return_by_val");
        std::printf("CASE 14: char_global_return_by_val value=%d\n", (int)(g_var_char));
    }

    // Case 15: char_static_local_assign
    {
        static char s_var_char; s_var_char = 'A';
        s_var_char = 'B';
        check(s_var_char == 'B', "char_static_local_assign");
        std::printf("CASE 15: char_static_local_assign value=%d\n", (int)(s_var_char));
    }

    // Case 16: char_static_local_compound_assign
    {
        static char s_var_char; s_var_char = 'A';
        s_var_char += 1;
        check(s_var_char == ('A') + 1, "char_static_local_compound_assign");
        std::printf("CASE 16: char_static_local_compound_assign value=%d\n", (int)(s_var_char));
    }

    // Case 17: char_static_local_inc_dec
    {
        static char s_var_char; s_var_char = 'A';
        ++s_var_char;
        --s_var_char;
        check(s_var_char == 'A', "char_static_local_inc_dec");
        std::printf("CASE 17: char_static_local_inc_dec value=%d\n", (int)(s_var_char));
    }

    // Case 18: char_static_local_read_after_write
    {
        static char s_var_char; s_var_char = 'A';
        s_var_char = 'B';
        char read_val = s_var_char;
        check(read_val == 'B', "char_static_local_read_after_write");
        std::printf("CASE 18: char_static_local_read_after_write value=%d\n", (int)(s_var_char));
    }

    // Case 19: char_static_local_pass_by_val
    {
        static char s_var_char; s_var_char = 'A';
        take_val_g_char(s_var_char);
        check(s_var_char == 'A', "char_static_local_pass_by_val");
        std::printf("CASE 19: char_static_local_pass_by_val value=%d\n", (int)(s_var_char));
    }

    // Case 20: char_static_local_pass_by_ref
    {
        static char s_var_char; s_var_char = 'A';
        take_ref_g_char(s_var_char);
        check(s_var_char == 'A', "char_static_local_pass_by_ref");
        std::printf("CASE 20: char_static_local_pass_by_ref value=%d\n", (int)(s_var_char));
    }

    // Case 21: char_static_local_return_by_val
    {
        static char s_var_char; s_var_char = 'A';
        s_var_char = return_g_char();
        check(s_var_char == 'B', "char_static_local_return_by_val");
        std::printf("CASE 21: char_static_local_return_by_val value=%d\n", (int)(s_var_char));
    }

    // Case 22: char_struct_member_assign
    {
        Holder_g_char h; h.m = 'A';
        h.m = 'B';
        check(h.m == 'B', "char_struct_member_assign");
        std::printf("CASE 22: char_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 23: char_struct_member_compound_assign
    {
        Holder_g_char h; h.m = 'A';
        h.m += 1;
        check(h.m == ('A') + 1, "char_struct_member_compound_assign");
        std::printf("CASE 23: char_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 24: char_struct_member_inc_dec
    {
        Holder_g_char h; h.m = 'A';
        ++h.m;
        --h.m;
        check(h.m == 'A', "char_struct_member_inc_dec");
        std::printf("CASE 24: char_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 25: char_struct_member_read_after_write
    {
        Holder_g_char h; h.m = 'A';
        h.m = 'B';
        char read_val = h.m;
        check(read_val == 'B', "char_struct_member_read_after_write");
        std::printf("CASE 25: char_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 26: char_struct_member_pass_by_val
    {
        Holder_g_char h; h.m = 'A';
        take_val_g_char(h.m);
        check(h.m == 'A', "char_struct_member_pass_by_val");
        std::printf("CASE 26: char_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 27: char_struct_member_pass_by_ref
    {
        Holder_g_char h; h.m = 'A';
        take_ref_g_char(h.m);
        check(h.m == 'A', "char_struct_member_pass_by_ref");
        std::printf("CASE 27: char_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 28: char_struct_member_return_by_val
    {
        Holder_g_char h; h.m = 'A';
        h.m = return_g_char();
        check(h.m == 'B', "char_struct_member_return_by_val");
        std::printf("CASE 28: char_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 29: char_array_elem_assign
    {
        char arr[3] = { 'A', 'A', 'A' };
        arr[1] = 'B';
        check(arr[1] == 'B', "char_array_elem_assign");
        std::printf("CASE 29: char_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 30: char_array_elem_compound_assign
    {
        char arr[3] = { 'A', 'A', 'A' };
        arr[1] += 1;
        check(arr[1] == ('A') + 1, "char_array_elem_compound_assign");
        std::printf("CASE 30: char_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 31: char_array_elem_inc_dec
    {
        char arr[3] = { 'A', 'A', 'A' };
        ++arr[1];
        --arr[1];
        check(arr[1] == 'A', "char_array_elem_inc_dec");
        std::printf("CASE 31: char_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 32: char_array_elem_read_after_write
    {
        char arr[3] = { 'A', 'A', 'A' };
        arr[1] = 'B';
        char read_val = arr[1];
        check(read_val == 'B', "char_array_elem_read_after_write");
        std::printf("CASE 32: char_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 33: char_array_elem_pass_by_val
    {
        char arr[3] = { 'A', 'A', 'A' };
        take_val_g_char(arr[1]);
        check(arr[1] == 'A', "char_array_elem_pass_by_val");
        std::printf("CASE 33: char_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 34: char_array_elem_pass_by_ref
    {
        char arr[3] = { 'A', 'A', 'A' };
        take_ref_g_char(arr[1]);
        check(arr[1] == 'A', "char_array_elem_pass_by_ref");
        std::printf("CASE 34: char_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 35: char_array_elem_return_by_val
    {
        char arr[3] = { 'A', 'A', 'A' };
        arr[1] = return_g_char();
        check(arr[1] == 'B', "char_array_elem_return_by_val");
        std::printf("CASE 35: char_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 36: char_via_ref_assign
    {
        char base_val = 'A';
        char& ref = base_val;
        ref = 'B';
        check(ref == 'B', "char_via_ref_assign");
        std::printf("CASE 36: char_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 37: char_via_ref_compound_assign
    {
        char base_val = 'A';
        char& ref = base_val;
        ref += 1;
        check(ref == ('A') + 1, "char_via_ref_compound_assign");
        std::printf("CASE 37: char_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 38: char_via_ref_inc_dec
    {
        char base_val = 'A';
        char& ref = base_val;
        ++ref;
        --ref;
        check(ref == 'A', "char_via_ref_inc_dec");
        std::printf("CASE 38: char_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 39: char_via_ref_read_after_write
    {
        char base_val = 'A';
        char& ref = base_val;
        ref = 'B';
        char read_val = ref;
        check(read_val == 'B', "char_via_ref_read_after_write");
        std::printf("CASE 39: char_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 40: char_via_ref_pass_by_val
    {
        char base_val = 'A';
        char& ref = base_val;
        take_val_g_char(ref);
        check(ref == 'A', "char_via_ref_pass_by_val");
        std::printf("CASE 40: char_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 41: char_via_ref_pass_by_ref
    {
        char base_val = 'A';
        char& ref = base_val;
        take_ref_g_char(ref);
        check(ref == 'A', "char_via_ref_pass_by_ref");
        std::printf("CASE 41: char_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 42: char_via_ref_return_by_val
    {
        char base_val = 'A';
        char& ref = base_val;
        ref = return_g_char();
        check(ref == 'B', "char_via_ref_return_by_val");
        std::printf("CASE 42: char_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 43: char_via_ptr_assign
    {
        char base_val = 'A';
        char* ptr = &base_val;
        *ptr = 'B';
        check(*ptr == 'B', "char_via_ptr_assign");
        std::printf("CASE 43: char_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 44: char_via_ptr_compound_assign
    {
        char base_val = 'A';
        char* ptr = &base_val;
        *ptr += 1;
        check(*ptr == ('A') + 1, "char_via_ptr_compound_assign");
        std::printf("CASE 44: char_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 45: char_via_ptr_inc_dec
    {
        char base_val = 'A';
        char* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 'A', "char_via_ptr_inc_dec");
        std::printf("CASE 45: char_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 46: char_via_ptr_read_after_write
    {
        char base_val = 'A';
        char* ptr = &base_val;
        *ptr = 'B';
        char read_val = *ptr;
        check(read_val == 'B', "char_via_ptr_read_after_write");
        std::printf("CASE 46: char_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 47: char_via_ptr_pass_by_val
    {
        char base_val = 'A';
        char* ptr = &base_val;
        take_val_g_char(*ptr);
        check(*ptr == 'A', "char_via_ptr_pass_by_val");
        std::printf("CASE 47: char_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 48: char_via_ptr_pass_by_ref
    {
        char base_val = 'A';
        char* ptr = &base_val;
        take_ref_g_char(*ptr);
        check(*ptr == 'A', "char_via_ptr_pass_by_ref");
        std::printf("CASE 48: char_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 49: char_via_ptr_return_by_val
    {
        char base_val = 'A';
        char* ptr = &base_val;
        *ptr = return_g_char();
        check(*ptr == 'B', "char_via_ptr_return_by_val");
        std::printf("CASE 49: char_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 50: char_via_pptr_assign
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr = 'B';
        check(**pptr == 'B', "char_via_pptr_assign");
        std::printf("CASE 50: char_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 51: char_via_pptr_compound_assign
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr += 1;
        check(**pptr == ('A') + 1, "char_via_pptr_compound_assign");
        std::printf("CASE 51: char_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 52: char_via_pptr_inc_dec
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 'A', "char_via_pptr_inc_dec");
        std::printf("CASE 52: char_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 53: char_via_pptr_read_after_write
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr = 'B';
        char read_val = **pptr;
        check(read_val == 'B', "char_via_pptr_read_after_write");
        std::printf("CASE 53: char_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 54: char_via_pptr_pass_by_val
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        take_val_g_char(**pptr);
        check(**pptr == 'A', "char_via_pptr_pass_by_val");
        std::printf("CASE 54: char_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 55: char_via_pptr_pass_by_ref
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        take_ref_g_char(**pptr);
        check(**pptr == 'A', "char_via_pptr_pass_by_ref");
        std::printf("CASE 55: char_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 56: char_via_pptr_return_by_val
    {
        char base_val = 'A';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr = return_g_char();
        check(**pptr == 'B', "char_via_pptr_return_by_val");
        std::printf("CASE 56: char_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 57: short_local_assign
    {
        short lv = 12;
        lv = 34;
        check(lv == 34, "short_local_assign");
        std::printf("CASE 57: short_local_assign value=%d\n", (int)(lv));
    }

    // Case 58: short_local_compound_assign
    {
        short lv = 12;
        lv += 5;
        check(lv == (12) + 5, "short_local_compound_assign");
        std::printf("CASE 58: short_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 59: short_local_inc_dec
    {
        short lv = 12;
        ++lv;
        --lv;
        check(lv == 12, "short_local_inc_dec");
        std::printf("CASE 59: short_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 60: short_local_read_after_write
    {
        short lv = 12;
        lv = 34;
        short read_val = lv;
        check(read_val == 34, "short_local_read_after_write");
        std::printf("CASE 60: short_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 61: short_local_pass_by_val
    {
        short lv = 12;
        take_val_g_short(lv);
        check(lv == 12, "short_local_pass_by_val");
        std::printf("CASE 61: short_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 62: short_local_pass_by_ref
    {
        short lv = 12;
        take_ref_g_short(lv);
        check(lv == 12, "short_local_pass_by_ref");
        std::printf("CASE 62: short_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 63: short_local_return_by_val
    {
        short lv = 12;
        lv = return_g_short();
        check(lv == 34, "short_local_return_by_val");
        std::printf("CASE 63: short_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 64: short_global_assign
    {
        g_var_short = 12;
        g_var_short = 34;
        check(g_var_short == 34, "short_global_assign");
        std::printf("CASE 64: short_global_assign value=%d\n", (int)(g_var_short));
    }

    // Case 65: short_global_compound_assign
    {
        g_var_short = 12;
        g_var_short += 5;
        check(g_var_short == (12) + 5, "short_global_compound_assign");
        std::printf("CASE 65: short_global_compound_assign value=%d\n", (int)(g_var_short));
    }

    // Case 66: short_global_inc_dec
    {
        g_var_short = 12;
        ++g_var_short;
        --g_var_short;
        check(g_var_short == 12, "short_global_inc_dec");
        std::printf("CASE 66: short_global_inc_dec value=%d\n", (int)(g_var_short));
    }

    // Case 67: short_global_read_after_write
    {
        g_var_short = 12;
        g_var_short = 34;
        short read_val = g_var_short;
        check(read_val == 34, "short_global_read_after_write");
        std::printf("CASE 67: short_global_read_after_write value=%d\n", (int)(g_var_short));
    }

    // Case 68: short_global_pass_by_val
    {
        g_var_short = 12;
        take_val_g_short(g_var_short);
        check(g_var_short == 12, "short_global_pass_by_val");
        std::printf("CASE 68: short_global_pass_by_val value=%d\n", (int)(g_var_short));
    }

    // Case 69: short_global_pass_by_ref
    {
        g_var_short = 12;
        take_ref_g_short(g_var_short);
        check(g_var_short == 12, "short_global_pass_by_ref");
        std::printf("CASE 69: short_global_pass_by_ref value=%d\n", (int)(g_var_short));
    }

    // Case 70: short_global_return_by_val
    {
        g_var_short = 12;
        g_var_short = return_g_short();
        check(g_var_short == 34, "short_global_return_by_val");
        std::printf("CASE 70: short_global_return_by_val value=%d\n", (int)(g_var_short));
    }

    // Case 71: short_static_local_assign
    {
        static short s_var_short; s_var_short = 12;
        s_var_short = 34;
        check(s_var_short == 34, "short_static_local_assign");
        std::printf("CASE 71: short_static_local_assign value=%d\n", (int)(s_var_short));
    }

    // Case 72: short_static_local_compound_assign
    {
        static short s_var_short; s_var_short = 12;
        s_var_short += 5;
        check(s_var_short == (12) + 5, "short_static_local_compound_assign");
        std::printf("CASE 72: short_static_local_compound_assign value=%d\n", (int)(s_var_short));
    }

    // Case 73: short_static_local_inc_dec
    {
        static short s_var_short; s_var_short = 12;
        ++s_var_short;
        --s_var_short;
        check(s_var_short == 12, "short_static_local_inc_dec");
        std::printf("CASE 73: short_static_local_inc_dec value=%d\n", (int)(s_var_short));
    }

    // Case 74: short_static_local_read_after_write
    {
        static short s_var_short; s_var_short = 12;
        s_var_short = 34;
        short read_val = s_var_short;
        check(read_val == 34, "short_static_local_read_after_write");
        std::printf("CASE 74: short_static_local_read_after_write value=%d\n", (int)(s_var_short));
    }

    // Case 75: short_static_local_pass_by_val
    {
        static short s_var_short; s_var_short = 12;
        take_val_g_short(s_var_short);
        check(s_var_short == 12, "short_static_local_pass_by_val");
        std::printf("CASE 75: short_static_local_pass_by_val value=%d\n", (int)(s_var_short));
    }

    // Case 76: short_static_local_pass_by_ref
    {
        static short s_var_short; s_var_short = 12;
        take_ref_g_short(s_var_short);
        check(s_var_short == 12, "short_static_local_pass_by_ref");
        std::printf("CASE 76: short_static_local_pass_by_ref value=%d\n", (int)(s_var_short));
    }

    // Case 77: short_static_local_return_by_val
    {
        static short s_var_short; s_var_short = 12;
        s_var_short = return_g_short();
        check(s_var_short == 34, "short_static_local_return_by_val");
        std::printf("CASE 77: short_static_local_return_by_val value=%d\n", (int)(s_var_short));
    }

    // Case 78: short_struct_member_assign
    {
        Holder_g_short h; h.m = 12;
        h.m = 34;
        check(h.m == 34, "short_struct_member_assign");
        std::printf("CASE 78: short_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 79: short_struct_member_compound_assign
    {
        Holder_g_short h; h.m = 12;
        h.m += 5;
        check(h.m == (12) + 5, "short_struct_member_compound_assign");
        std::printf("CASE 79: short_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 80: short_struct_member_inc_dec
    {
        Holder_g_short h; h.m = 12;
        ++h.m;
        --h.m;
        check(h.m == 12, "short_struct_member_inc_dec");
        std::printf("CASE 80: short_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 81: short_struct_member_read_after_write
    {
        Holder_g_short h; h.m = 12;
        h.m = 34;
        short read_val = h.m;
        check(read_val == 34, "short_struct_member_read_after_write");
        std::printf("CASE 81: short_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 82: short_struct_member_pass_by_val
    {
        Holder_g_short h; h.m = 12;
        take_val_g_short(h.m);
        check(h.m == 12, "short_struct_member_pass_by_val");
        std::printf("CASE 82: short_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 83: short_struct_member_pass_by_ref
    {
        Holder_g_short h; h.m = 12;
        take_ref_g_short(h.m);
        check(h.m == 12, "short_struct_member_pass_by_ref");
        std::printf("CASE 83: short_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 84: short_struct_member_return_by_val
    {
        Holder_g_short h; h.m = 12;
        h.m = return_g_short();
        check(h.m == 34, "short_struct_member_return_by_val");
        std::printf("CASE 84: short_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 85: short_array_elem_assign
    {
        short arr[3] = { 12, 12, 12 };
        arr[1] = 34;
        check(arr[1] == 34, "short_array_elem_assign");
        std::printf("CASE 85: short_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 86: short_array_elem_compound_assign
    {
        short arr[3] = { 12, 12, 12 };
        arr[1] += 5;
        check(arr[1] == (12) + 5, "short_array_elem_compound_assign");
        std::printf("CASE 86: short_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 87: short_array_elem_inc_dec
    {
        short arr[3] = { 12, 12, 12 };
        ++arr[1];
        --arr[1];
        check(arr[1] == 12, "short_array_elem_inc_dec");
        std::printf("CASE 87: short_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 88: short_array_elem_read_after_write
    {
        short arr[3] = { 12, 12, 12 };
        arr[1] = 34;
        short read_val = arr[1];
        check(read_val == 34, "short_array_elem_read_after_write");
        std::printf("CASE 88: short_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 89: short_array_elem_pass_by_val
    {
        short arr[3] = { 12, 12, 12 };
        take_val_g_short(arr[1]);
        check(arr[1] == 12, "short_array_elem_pass_by_val");
        std::printf("CASE 89: short_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 90: short_array_elem_pass_by_ref
    {
        short arr[3] = { 12, 12, 12 };
        take_ref_g_short(arr[1]);
        check(arr[1] == 12, "short_array_elem_pass_by_ref");
        std::printf("CASE 90: short_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 91: short_array_elem_return_by_val
    {
        short arr[3] = { 12, 12, 12 };
        arr[1] = return_g_short();
        check(arr[1] == 34, "short_array_elem_return_by_val");
        std::printf("CASE 91: short_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 92: short_via_ref_assign
    {
        short base_val = 12;
        short& ref = base_val;
        ref = 34;
        check(ref == 34, "short_via_ref_assign");
        std::printf("CASE 92: short_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 93: short_via_ref_compound_assign
    {
        short base_val = 12;
        short& ref = base_val;
        ref += 5;
        check(ref == (12) + 5, "short_via_ref_compound_assign");
        std::printf("CASE 93: short_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 94: short_via_ref_inc_dec
    {
        short base_val = 12;
        short& ref = base_val;
        ++ref;
        --ref;
        check(ref == 12, "short_via_ref_inc_dec");
        std::printf("CASE 94: short_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 95: short_via_ref_read_after_write
    {
        short base_val = 12;
        short& ref = base_val;
        ref = 34;
        short read_val = ref;
        check(read_val == 34, "short_via_ref_read_after_write");
        std::printf("CASE 95: short_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 96: short_via_ref_pass_by_val
    {
        short base_val = 12;
        short& ref = base_val;
        take_val_g_short(ref);
        check(ref == 12, "short_via_ref_pass_by_val");
        std::printf("CASE 96: short_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 97: short_via_ref_pass_by_ref
    {
        short base_val = 12;
        short& ref = base_val;
        take_ref_g_short(ref);
        check(ref == 12, "short_via_ref_pass_by_ref");
        std::printf("CASE 97: short_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 98: short_via_ref_return_by_val
    {
        short base_val = 12;
        short& ref = base_val;
        ref = return_g_short();
        check(ref == 34, "short_via_ref_return_by_val");
        std::printf("CASE 98: short_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 99: short_via_ptr_assign
    {
        short base_val = 12;
        short* ptr = &base_val;
        *ptr = 34;
        check(*ptr == 34, "short_via_ptr_assign");
        std::printf("CASE 99: short_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 100: short_via_ptr_compound_assign
    {
        short base_val = 12;
        short* ptr = &base_val;
        *ptr += 5;
        check(*ptr == (12) + 5, "short_via_ptr_compound_assign");
        std::printf("CASE 100: short_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 101: short_via_ptr_inc_dec
    {
        short base_val = 12;
        short* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 12, "short_via_ptr_inc_dec");
        std::printf("CASE 101: short_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 102: short_via_ptr_read_after_write
    {
        short base_val = 12;
        short* ptr = &base_val;
        *ptr = 34;
        short read_val = *ptr;
        check(read_val == 34, "short_via_ptr_read_after_write");
        std::printf("CASE 102: short_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 103: short_via_ptr_pass_by_val
    {
        short base_val = 12;
        short* ptr = &base_val;
        take_val_g_short(*ptr);
        check(*ptr == 12, "short_via_ptr_pass_by_val");
        std::printf("CASE 103: short_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 104: short_via_ptr_pass_by_ref
    {
        short base_val = 12;
        short* ptr = &base_val;
        take_ref_g_short(*ptr);
        check(*ptr == 12, "short_via_ptr_pass_by_ref");
        std::printf("CASE 104: short_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 105: short_via_ptr_return_by_val
    {
        short base_val = 12;
        short* ptr = &base_val;
        *ptr = return_g_short();
        check(*ptr == 34, "short_via_ptr_return_by_val");
        std::printf("CASE 105: short_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 106: short_via_pptr_assign
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr = 34;
        check(**pptr == 34, "short_via_pptr_assign");
        std::printf("CASE 106: short_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 107: short_via_pptr_compound_assign
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr += 5;
        check(**pptr == (12) + 5, "short_via_pptr_compound_assign");
        std::printf("CASE 107: short_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 108: short_via_pptr_inc_dec
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 12, "short_via_pptr_inc_dec");
        std::printf("CASE 108: short_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 109: short_via_pptr_read_after_write
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr = 34;
        short read_val = **pptr;
        check(read_val == 34, "short_via_pptr_read_after_write");
        std::printf("CASE 109: short_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 110: short_via_pptr_pass_by_val
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        take_val_g_short(**pptr);
        check(**pptr == 12, "short_via_pptr_pass_by_val");
        std::printf("CASE 110: short_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 111: short_via_pptr_pass_by_ref
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        take_ref_g_short(**pptr);
        check(**pptr == 12, "short_via_pptr_pass_by_ref");
        std::printf("CASE 111: short_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 112: short_via_pptr_return_by_val
    {
        short base_val = 12;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr = return_g_short();
        check(**pptr == 34, "short_via_pptr_return_by_val");
        std::printf("CASE 112: short_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 113: int_local_assign
    {
        int lv = 101;
        lv = 202;
        check(lv == 202, "int_local_assign");
        std::printf("CASE 113: int_local_assign value=%d\n", (int)(lv));
    }

    // Case 114: int_local_compound_assign
    {
        int lv = 101;
        lv += 7;
        check(lv == (101) + 7, "int_local_compound_assign");
        std::printf("CASE 114: int_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 115: int_local_inc_dec
    {
        int lv = 101;
        ++lv;
        --lv;
        check(lv == 101, "int_local_inc_dec");
        std::printf("CASE 115: int_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 116: int_local_read_after_write
    {
        int lv = 101;
        lv = 202;
        int read_val = lv;
        check(read_val == 202, "int_local_read_after_write");
        std::printf("CASE 116: int_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 117: int_local_pass_by_val
    {
        int lv = 101;
        take_val_g_int(lv);
        check(lv == 101, "int_local_pass_by_val");
        std::printf("CASE 117: int_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 118: int_local_pass_by_ref
    {
        int lv = 101;
        take_ref_g_int(lv);
        check(lv == 101, "int_local_pass_by_ref");
        std::printf("CASE 118: int_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 119: int_local_return_by_val
    {
        int lv = 101;
        lv = return_g_int();
        check(lv == 202, "int_local_return_by_val");
        std::printf("CASE 119: int_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 120: int_global_assign
    {
        g_var_int = 101;
        g_var_int = 202;
        check(g_var_int == 202, "int_global_assign");
        std::printf("CASE 120: int_global_assign value=%d\n", (int)(g_var_int));
    }

    // Case 121: int_global_compound_assign
    {
        g_var_int = 101;
        g_var_int += 7;
        check(g_var_int == (101) + 7, "int_global_compound_assign");
        std::printf("CASE 121: int_global_compound_assign value=%d\n", (int)(g_var_int));
    }

    // Case 122: int_global_inc_dec
    {
        g_var_int = 101;
        ++g_var_int;
        --g_var_int;
        check(g_var_int == 101, "int_global_inc_dec");
        std::printf("CASE 122: int_global_inc_dec value=%d\n", (int)(g_var_int));
    }

    // Case 123: int_global_read_after_write
    {
        g_var_int = 101;
        g_var_int = 202;
        int read_val = g_var_int;
        check(read_val == 202, "int_global_read_after_write");
        std::printf("CASE 123: int_global_read_after_write value=%d\n", (int)(g_var_int));
    }

    // Case 124: int_global_pass_by_val
    {
        g_var_int = 101;
        take_val_g_int(g_var_int);
        check(g_var_int == 101, "int_global_pass_by_val");
        std::printf("CASE 124: int_global_pass_by_val value=%d\n", (int)(g_var_int));
    }

    // Case 125: int_global_pass_by_ref
    {
        g_var_int = 101;
        take_ref_g_int(g_var_int);
        check(g_var_int == 101, "int_global_pass_by_ref");
        std::printf("CASE 125: int_global_pass_by_ref value=%d\n", (int)(g_var_int));
    }

    // Case 126: int_global_return_by_val
    {
        g_var_int = 101;
        g_var_int = return_g_int();
        check(g_var_int == 202, "int_global_return_by_val");
        std::printf("CASE 126: int_global_return_by_val value=%d\n", (int)(g_var_int));
    }

    // Case 127: int_static_local_assign
    {
        static int s_var_int; s_var_int = 101;
        s_var_int = 202;
        check(s_var_int == 202, "int_static_local_assign");
        std::printf("CASE 127: int_static_local_assign value=%d\n", (int)(s_var_int));
    }

    // Case 128: int_static_local_compound_assign
    {
        static int s_var_int; s_var_int = 101;
        s_var_int += 7;
        check(s_var_int == (101) + 7, "int_static_local_compound_assign");
        std::printf("CASE 128: int_static_local_compound_assign value=%d\n", (int)(s_var_int));
    }

    // Case 129: int_static_local_inc_dec
    {
        static int s_var_int; s_var_int = 101;
        ++s_var_int;
        --s_var_int;
        check(s_var_int == 101, "int_static_local_inc_dec");
        std::printf("CASE 129: int_static_local_inc_dec value=%d\n", (int)(s_var_int));
    }

    // Case 130: int_static_local_read_after_write
    {
        static int s_var_int; s_var_int = 101;
        s_var_int = 202;
        int read_val = s_var_int;
        check(read_val == 202, "int_static_local_read_after_write");
        std::printf("CASE 130: int_static_local_read_after_write value=%d\n", (int)(s_var_int));
    }

    // Case 131: int_static_local_pass_by_val
    {
        static int s_var_int; s_var_int = 101;
        take_val_g_int(s_var_int);
        check(s_var_int == 101, "int_static_local_pass_by_val");
        std::printf("CASE 131: int_static_local_pass_by_val value=%d\n", (int)(s_var_int));
    }

    // Case 132: int_static_local_pass_by_ref
    {
        static int s_var_int; s_var_int = 101;
        take_ref_g_int(s_var_int);
        check(s_var_int == 101, "int_static_local_pass_by_ref");
        std::printf("CASE 132: int_static_local_pass_by_ref value=%d\n", (int)(s_var_int));
    }

    // Case 133: int_static_local_return_by_val
    {
        static int s_var_int; s_var_int = 101;
        s_var_int = return_g_int();
        check(s_var_int == 202, "int_static_local_return_by_val");
        std::printf("CASE 133: int_static_local_return_by_val value=%d\n", (int)(s_var_int));
    }

    // Case 134: int_struct_member_assign
    {
        Holder_g_int h; h.m = 101;
        h.m = 202;
        check(h.m == 202, "int_struct_member_assign");
        std::printf("CASE 134: int_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 135: int_struct_member_compound_assign
    {
        Holder_g_int h; h.m = 101;
        h.m += 7;
        check(h.m == (101) + 7, "int_struct_member_compound_assign");
        std::printf("CASE 135: int_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 136: int_struct_member_inc_dec
    {
        Holder_g_int h; h.m = 101;
        ++h.m;
        --h.m;
        check(h.m == 101, "int_struct_member_inc_dec");
        std::printf("CASE 136: int_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 137: int_struct_member_read_after_write
    {
        Holder_g_int h; h.m = 101;
        h.m = 202;
        int read_val = h.m;
        check(read_val == 202, "int_struct_member_read_after_write");
        std::printf("CASE 137: int_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 138: int_struct_member_pass_by_val
    {
        Holder_g_int h; h.m = 101;
        take_val_g_int(h.m);
        check(h.m == 101, "int_struct_member_pass_by_val");
        std::printf("CASE 138: int_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 139: int_struct_member_pass_by_ref
    {
        Holder_g_int h; h.m = 101;
        take_ref_g_int(h.m);
        check(h.m == 101, "int_struct_member_pass_by_ref");
        std::printf("CASE 139: int_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 140: int_struct_member_return_by_val
    {
        Holder_g_int h; h.m = 101;
        h.m = return_g_int();
        check(h.m == 202, "int_struct_member_return_by_val");
        std::printf("CASE 140: int_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 141: int_array_elem_assign
    {
        int arr[3] = { 101, 101, 101 };
        arr[1] = 202;
        check(arr[1] == 202, "int_array_elem_assign");
        std::printf("CASE 141: int_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 142: int_array_elem_compound_assign
    {
        int arr[3] = { 101, 101, 101 };
        arr[1] += 7;
        check(arr[1] == (101) + 7, "int_array_elem_compound_assign");
        std::printf("CASE 142: int_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 143: int_array_elem_inc_dec
    {
        int arr[3] = { 101, 101, 101 };
        ++arr[1];
        --arr[1];
        check(arr[1] == 101, "int_array_elem_inc_dec");
        std::printf("CASE 143: int_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 144: int_array_elem_read_after_write
    {
        int arr[3] = { 101, 101, 101 };
        arr[1] = 202;
        int read_val = arr[1];
        check(read_val == 202, "int_array_elem_read_after_write");
        std::printf("CASE 144: int_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 145: int_array_elem_pass_by_val
    {
        int arr[3] = { 101, 101, 101 };
        take_val_g_int(arr[1]);
        check(arr[1] == 101, "int_array_elem_pass_by_val");
        std::printf("CASE 145: int_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 146: int_array_elem_pass_by_ref
    {
        int arr[3] = { 101, 101, 101 };
        take_ref_g_int(arr[1]);
        check(arr[1] == 101, "int_array_elem_pass_by_ref");
        std::printf("CASE 146: int_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 147: int_array_elem_return_by_val
    {
        int arr[3] = { 101, 101, 101 };
        arr[1] = return_g_int();
        check(arr[1] == 202, "int_array_elem_return_by_val");
        std::printf("CASE 147: int_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 148: int_via_ref_assign
    {
        int base_val = 101;
        int& ref = base_val;
        ref = 202;
        check(ref == 202, "int_via_ref_assign");
        std::printf("CASE 148: int_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 149: int_via_ref_compound_assign
    {
        int base_val = 101;
        int& ref = base_val;
        ref += 7;
        check(ref == (101) + 7, "int_via_ref_compound_assign");
        std::printf("CASE 149: int_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 150: int_via_ref_inc_dec
    {
        int base_val = 101;
        int& ref = base_val;
        ++ref;
        --ref;
        check(ref == 101, "int_via_ref_inc_dec");
        std::printf("CASE 150: int_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 151: int_via_ref_read_after_write
    {
        int base_val = 101;
        int& ref = base_val;
        ref = 202;
        int read_val = ref;
        check(read_val == 202, "int_via_ref_read_after_write");
        std::printf("CASE 151: int_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 152: int_via_ref_pass_by_val
    {
        int base_val = 101;
        int& ref = base_val;
        take_val_g_int(ref);
        check(ref == 101, "int_via_ref_pass_by_val");
        std::printf("CASE 152: int_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 153: int_via_ref_pass_by_ref
    {
        int base_val = 101;
        int& ref = base_val;
        take_ref_g_int(ref);
        check(ref == 101, "int_via_ref_pass_by_ref");
        std::printf("CASE 153: int_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 154: int_via_ref_return_by_val
    {
        int base_val = 101;
        int& ref = base_val;
        ref = return_g_int();
        check(ref == 202, "int_via_ref_return_by_val");
        std::printf("CASE 154: int_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 155: int_via_ptr_assign
    {
        int base_val = 101;
        int* ptr = &base_val;
        *ptr = 202;
        check(*ptr == 202, "int_via_ptr_assign");
        std::printf("CASE 155: int_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 156: int_via_ptr_compound_assign
    {
        int base_val = 101;
        int* ptr = &base_val;
        *ptr += 7;
        check(*ptr == (101) + 7, "int_via_ptr_compound_assign");
        std::printf("CASE 156: int_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 157: int_via_ptr_inc_dec
    {
        int base_val = 101;
        int* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 101, "int_via_ptr_inc_dec");
        std::printf("CASE 157: int_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 158: int_via_ptr_read_after_write
    {
        int base_val = 101;
        int* ptr = &base_val;
        *ptr = 202;
        int read_val = *ptr;
        check(read_val == 202, "int_via_ptr_read_after_write");
        std::printf("CASE 158: int_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 159: int_via_ptr_pass_by_val
    {
        int base_val = 101;
        int* ptr = &base_val;
        take_val_g_int(*ptr);
        check(*ptr == 101, "int_via_ptr_pass_by_val");
        std::printf("CASE 159: int_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 160: int_via_ptr_pass_by_ref
    {
        int base_val = 101;
        int* ptr = &base_val;
        take_ref_g_int(*ptr);
        check(*ptr == 101, "int_via_ptr_pass_by_ref");
        std::printf("CASE 160: int_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 161: int_via_ptr_return_by_val
    {
        int base_val = 101;
        int* ptr = &base_val;
        *ptr = return_g_int();
        check(*ptr == 202, "int_via_ptr_return_by_val");
        std::printf("CASE 161: int_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 162: int_via_pptr_assign
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr = 202;
        check(**pptr == 202, "int_via_pptr_assign");
        std::printf("CASE 162: int_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 163: int_via_pptr_compound_assign
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr += 7;
        check(**pptr == (101) + 7, "int_via_pptr_compound_assign");
        std::printf("CASE 163: int_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 164: int_via_pptr_inc_dec
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 101, "int_via_pptr_inc_dec");
        std::printf("CASE 164: int_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 165: int_via_pptr_read_after_write
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr = 202;
        int read_val = **pptr;
        check(read_val == 202, "int_via_pptr_read_after_write");
        std::printf("CASE 165: int_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 166: int_via_pptr_pass_by_val
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        take_val_g_int(**pptr);
        check(**pptr == 101, "int_via_pptr_pass_by_val");
        std::printf("CASE 166: int_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 167: int_via_pptr_pass_by_ref
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        take_ref_g_int(**pptr);
        check(**pptr == 101, "int_via_pptr_pass_by_ref");
        std::printf("CASE 167: int_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 168: int_via_pptr_return_by_val
    {
        int base_val = 101;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr = return_g_int();
        check(**pptr == 202, "int_via_pptr_return_by_val");
        std::printf("CASE 168: int_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 169: long_long_local_assign
    {
        long long lv = 1000000000LL;
        lv = 2000000000LL;
        check(lv == 2000000000LL, "long_long_local_assign");
        std::printf("CASE 169: long_long_local_assign value=%d\n", (int)(lv));
    }

    // Case 170: long_long_local_compound_assign
    {
        long long lv = 1000000000LL;
        lv += 100;
        check(lv == (1000000000LL) + 100, "long_long_local_compound_assign");
        std::printf("CASE 170: long_long_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 171: long_long_local_inc_dec
    {
        long long lv = 1000000000LL;
        ++lv;
        --lv;
        check(lv == 1000000000LL, "long_long_local_inc_dec");
        std::printf("CASE 171: long_long_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 172: long_long_local_read_after_write
    {
        long long lv = 1000000000LL;
        lv = 2000000000LL;
        long long read_val = lv;
        check(read_val == 2000000000LL, "long_long_local_read_after_write");
        std::printf("CASE 172: long_long_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 173: long_long_local_pass_by_val
    {
        long long lv = 1000000000LL;
        take_val_g_long_long(lv);
        check(lv == 1000000000LL, "long_long_local_pass_by_val");
        std::printf("CASE 173: long_long_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 174: long_long_local_pass_by_ref
    {
        long long lv = 1000000000LL;
        take_ref_g_long_long(lv);
        check(lv == 1000000000LL, "long_long_local_pass_by_ref");
        std::printf("CASE 174: long_long_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 175: long_long_local_return_by_val
    {
        long long lv = 1000000000LL;
        lv = return_g_long_long();
        check(lv == 2000000000LL, "long_long_local_return_by_val");
        std::printf("CASE 175: long_long_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 176: long_long_global_assign
    {
        g_var_long_long = 1000000000LL;
        g_var_long_long = 2000000000LL;
        check(g_var_long_long == 2000000000LL, "long_long_global_assign");
        std::printf("CASE 176: long_long_global_assign value=%d\n", (int)(g_var_long_long));
    }

    // Case 177: long_long_global_compound_assign
    {
        g_var_long_long = 1000000000LL;
        g_var_long_long += 100;
        check(g_var_long_long == (1000000000LL) + 100, "long_long_global_compound_assign");
        std::printf("CASE 177: long_long_global_compound_assign value=%d\n", (int)(g_var_long_long));
    }

    // Case 178: long_long_global_inc_dec
    {
        g_var_long_long = 1000000000LL;
        ++g_var_long_long;
        --g_var_long_long;
        check(g_var_long_long == 1000000000LL, "long_long_global_inc_dec");
        std::printf("CASE 178: long_long_global_inc_dec value=%d\n", (int)(g_var_long_long));
    }

    // Case 179: long_long_global_read_after_write
    {
        g_var_long_long = 1000000000LL;
        g_var_long_long = 2000000000LL;
        long long read_val = g_var_long_long;
        check(read_val == 2000000000LL, "long_long_global_read_after_write");
        std::printf("CASE 179: long_long_global_read_after_write value=%d\n", (int)(g_var_long_long));
    }

    // Case 180: long_long_global_pass_by_val
    {
        g_var_long_long = 1000000000LL;
        take_val_g_long_long(g_var_long_long);
        check(g_var_long_long == 1000000000LL, "long_long_global_pass_by_val");
        std::printf("CASE 180: long_long_global_pass_by_val value=%d\n", (int)(g_var_long_long));
    }

    // Case 181: long_long_global_pass_by_ref
    {
        g_var_long_long = 1000000000LL;
        take_ref_g_long_long(g_var_long_long);
        check(g_var_long_long == 1000000000LL, "long_long_global_pass_by_ref");
        std::printf("CASE 181: long_long_global_pass_by_ref value=%d\n", (int)(g_var_long_long));
    }

    // Case 182: long_long_global_return_by_val
    {
        g_var_long_long = 1000000000LL;
        g_var_long_long = return_g_long_long();
        check(g_var_long_long == 2000000000LL, "long_long_global_return_by_val");
        std::printf("CASE 182: long_long_global_return_by_val value=%d\n", (int)(g_var_long_long));
    }

    // Case 183: long_long_static_local_assign
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        s_var_long_long = 2000000000LL;
        check(s_var_long_long == 2000000000LL, "long_long_static_local_assign");
        std::printf("CASE 183: long_long_static_local_assign value=%d\n", (int)(s_var_long_long));
    }

    // Case 184: long_long_static_local_compound_assign
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        s_var_long_long += 100;
        check(s_var_long_long == (1000000000LL) + 100, "long_long_static_local_compound_assign");
        std::printf("CASE 184: long_long_static_local_compound_assign value=%d\n", (int)(s_var_long_long));
    }

    // Case 185: long_long_static_local_inc_dec
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        ++s_var_long_long;
        --s_var_long_long;
        check(s_var_long_long == 1000000000LL, "long_long_static_local_inc_dec");
        std::printf("CASE 185: long_long_static_local_inc_dec value=%d\n", (int)(s_var_long_long));
    }

    // Case 186: long_long_static_local_read_after_write
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        s_var_long_long = 2000000000LL;
        long long read_val = s_var_long_long;
        check(read_val == 2000000000LL, "long_long_static_local_read_after_write");
        std::printf("CASE 186: long_long_static_local_read_after_write value=%d\n", (int)(s_var_long_long));
    }

    // Case 187: long_long_static_local_pass_by_val
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        take_val_g_long_long(s_var_long_long);
        check(s_var_long_long == 1000000000LL, "long_long_static_local_pass_by_val");
        std::printf("CASE 187: long_long_static_local_pass_by_val value=%d\n", (int)(s_var_long_long));
    }

    // Case 188: long_long_static_local_pass_by_ref
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        take_ref_g_long_long(s_var_long_long);
        check(s_var_long_long == 1000000000LL, "long_long_static_local_pass_by_ref");
        std::printf("CASE 188: long_long_static_local_pass_by_ref value=%d\n", (int)(s_var_long_long));
    }

    // Case 189: long_long_static_local_return_by_val
    {
        static long long s_var_long_long; s_var_long_long = 1000000000LL;
        s_var_long_long = return_g_long_long();
        check(s_var_long_long == 2000000000LL, "long_long_static_local_return_by_val");
        std::printf("CASE 189: long_long_static_local_return_by_val value=%d\n", (int)(s_var_long_long));
    }

    // Case 190: long_long_struct_member_assign
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        h.m = 2000000000LL;
        check(h.m == 2000000000LL, "long_long_struct_member_assign");
        std::printf("CASE 190: long_long_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 191: long_long_struct_member_compound_assign
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        h.m += 100;
        check(h.m == (1000000000LL) + 100, "long_long_struct_member_compound_assign");
        std::printf("CASE 191: long_long_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 192: long_long_struct_member_inc_dec
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        ++h.m;
        --h.m;
        check(h.m == 1000000000LL, "long_long_struct_member_inc_dec");
        std::printf("CASE 192: long_long_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 193: long_long_struct_member_read_after_write
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        h.m = 2000000000LL;
        long long read_val = h.m;
        check(read_val == 2000000000LL, "long_long_struct_member_read_after_write");
        std::printf("CASE 193: long_long_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 194: long_long_struct_member_pass_by_val
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        take_val_g_long_long(h.m);
        check(h.m == 1000000000LL, "long_long_struct_member_pass_by_val");
        std::printf("CASE 194: long_long_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 195: long_long_struct_member_pass_by_ref
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        take_ref_g_long_long(h.m);
        check(h.m == 1000000000LL, "long_long_struct_member_pass_by_ref");
        std::printf("CASE 195: long_long_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 196: long_long_struct_member_return_by_val
    {
        Holder_g_long_long h; h.m = 1000000000LL;
        h.m = return_g_long_long();
        check(h.m == 2000000000LL, "long_long_struct_member_return_by_val");
        std::printf("CASE 196: long_long_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 197: long_long_array_elem_assign
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        arr[1] = 2000000000LL;
        check(arr[1] == 2000000000LL, "long_long_array_elem_assign");
        std::printf("CASE 197: long_long_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 198: long_long_array_elem_compound_assign
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        arr[1] += 100;
        check(arr[1] == (1000000000LL) + 100, "long_long_array_elem_compound_assign");
        std::printf("CASE 198: long_long_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 199: long_long_array_elem_inc_dec
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        ++arr[1];
        --arr[1];
        check(arr[1] == 1000000000LL, "long_long_array_elem_inc_dec");
        std::printf("CASE 199: long_long_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 200: long_long_array_elem_read_after_write
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        arr[1] = 2000000000LL;
        long long read_val = arr[1];
        check(read_val == 2000000000LL, "long_long_array_elem_read_after_write");
        std::printf("CASE 200: long_long_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 201: long_long_array_elem_pass_by_val
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        take_val_g_long_long(arr[1]);
        check(arr[1] == 1000000000LL, "long_long_array_elem_pass_by_val");
        std::printf("CASE 201: long_long_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 202: long_long_array_elem_pass_by_ref
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        take_ref_g_long_long(arr[1]);
        check(arr[1] == 1000000000LL, "long_long_array_elem_pass_by_ref");
        std::printf("CASE 202: long_long_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 203: long_long_array_elem_return_by_val
    {
        long long arr[3] = { 1000000000LL, 1000000000LL, 1000000000LL };
        arr[1] = return_g_long_long();
        check(arr[1] == 2000000000LL, "long_long_array_elem_return_by_val");
        std::printf("CASE 203: long_long_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 204: long_long_via_ref_assign
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        ref = 2000000000LL;
        check(ref == 2000000000LL, "long_long_via_ref_assign");
        std::printf("CASE 204: long_long_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 205: long_long_via_ref_compound_assign
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        ref += 100;
        check(ref == (1000000000LL) + 100, "long_long_via_ref_compound_assign");
        std::printf("CASE 205: long_long_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 206: long_long_via_ref_inc_dec
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        ++ref;
        --ref;
        check(ref == 1000000000LL, "long_long_via_ref_inc_dec");
        std::printf("CASE 206: long_long_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 207: long_long_via_ref_read_after_write
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        ref = 2000000000LL;
        long long read_val = ref;
        check(read_val == 2000000000LL, "long_long_via_ref_read_after_write");
        std::printf("CASE 207: long_long_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 208: long_long_via_ref_pass_by_val
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        take_val_g_long_long(ref);
        check(ref == 1000000000LL, "long_long_via_ref_pass_by_val");
        std::printf("CASE 208: long_long_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 209: long_long_via_ref_pass_by_ref
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        take_ref_g_long_long(ref);
        check(ref == 1000000000LL, "long_long_via_ref_pass_by_ref");
        std::printf("CASE 209: long_long_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 210: long_long_via_ref_return_by_val
    {
        long long base_val = 1000000000LL;
        long long& ref = base_val;
        ref = return_g_long_long();
        check(ref == 2000000000LL, "long_long_via_ref_return_by_val");
        std::printf("CASE 210: long_long_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 211: long_long_via_ptr_assign
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        *ptr = 2000000000LL;
        check(*ptr == 2000000000LL, "long_long_via_ptr_assign");
        std::printf("CASE 211: long_long_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 212: long_long_via_ptr_compound_assign
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        *ptr += 100;
        check(*ptr == (1000000000LL) + 100, "long_long_via_ptr_compound_assign");
        std::printf("CASE 212: long_long_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 213: long_long_via_ptr_inc_dec
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 1000000000LL, "long_long_via_ptr_inc_dec");
        std::printf("CASE 213: long_long_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 214: long_long_via_ptr_read_after_write
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        *ptr = 2000000000LL;
        long long read_val = *ptr;
        check(read_val == 2000000000LL, "long_long_via_ptr_read_after_write");
        std::printf("CASE 214: long_long_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 215: long_long_via_ptr_pass_by_val
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        take_val_g_long_long(*ptr);
        check(*ptr == 1000000000LL, "long_long_via_ptr_pass_by_val");
        std::printf("CASE 215: long_long_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 216: long_long_via_ptr_pass_by_ref
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        take_ref_g_long_long(*ptr);
        check(*ptr == 1000000000LL, "long_long_via_ptr_pass_by_ref");
        std::printf("CASE 216: long_long_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 217: long_long_via_ptr_return_by_val
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        *ptr = return_g_long_long();
        check(*ptr == 2000000000LL, "long_long_via_ptr_return_by_val");
        std::printf("CASE 217: long_long_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 218: long_long_via_pptr_assign
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr = 2000000000LL;
        check(**pptr == 2000000000LL, "long_long_via_pptr_assign");
        std::printf("CASE 218: long_long_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 219: long_long_via_pptr_compound_assign
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr += 100;
        check(**pptr == (1000000000LL) + 100, "long_long_via_pptr_compound_assign");
        std::printf("CASE 219: long_long_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 220: long_long_via_pptr_inc_dec
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 1000000000LL, "long_long_via_pptr_inc_dec");
        std::printf("CASE 220: long_long_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 221: long_long_via_pptr_read_after_write
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr = 2000000000LL;
        long long read_val = **pptr;
        check(read_val == 2000000000LL, "long_long_via_pptr_read_after_write");
        std::printf("CASE 221: long_long_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 222: long_long_via_pptr_pass_by_val
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        take_val_g_long_long(**pptr);
        check(**pptr == 1000000000LL, "long_long_via_pptr_pass_by_val");
        std::printf("CASE 222: long_long_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 223: long_long_via_pptr_pass_by_ref
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        take_ref_g_long_long(**pptr);
        check(**pptr == 1000000000LL, "long_long_via_pptr_pass_by_ref");
        std::printf("CASE 223: long_long_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 224: long_long_via_pptr_return_by_val
    {
        long long base_val = 1000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr = return_g_long_long();
        check(**pptr == 2000000000LL, "long_long_via_pptr_return_by_val");
        std::printf("CASE 224: long_long_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 225: float_local_assign
    {
        float lv = 1.5f;
        lv = 4.5f;
        check(lv == 4.5f, "float_local_assign");
        std::printf("CASE 225: float_local_assign value=%d\n", (int)(lv));
    }

    // Case 226: float_local_compound_assign
    {
        float lv = 1.5f;
        lv += 2.0;
        check(lv == (1.5f) + 2.0, "float_local_compound_assign");
        std::printf("CASE 226: float_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 227: float_local_inc_dec
    {
        float lv = 1.5f;
        ++lv;
        --lv;
        check(lv == 1.5f, "float_local_inc_dec");
        std::printf("CASE 227: float_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 228: float_local_read_after_write
    {
        float lv = 1.5f;
        lv = 4.5f;
        float read_val = lv;
        check(read_val == 4.5f, "float_local_read_after_write");
        std::printf("CASE 228: float_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 229: float_local_pass_by_val
    {
        float lv = 1.5f;
        take_val_g_float(lv);
        check(lv == 1.5f, "float_local_pass_by_val");
        std::printf("CASE 229: float_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 230: float_local_pass_by_ref
    {
        float lv = 1.5f;
        take_ref_g_float(lv);
        check(lv == 1.5f, "float_local_pass_by_ref");
        std::printf("CASE 230: float_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 231: float_local_return_by_val
    {
        float lv = 1.5f;
        lv = return_g_float();
        check(lv == 4.5f, "float_local_return_by_val");
        std::printf("CASE 231: float_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 232: float_global_assign
    {
        g_var_float = 1.5f;
        g_var_float = 4.5f;
        check(g_var_float == 4.5f, "float_global_assign");
        std::printf("CASE 232: float_global_assign value=%d\n", (int)(g_var_float));
    }

    // Case 233: float_global_compound_assign
    {
        g_var_float = 1.5f;
        g_var_float += 2.0;
        check(g_var_float == (1.5f) + 2.0, "float_global_compound_assign");
        std::printf("CASE 233: float_global_compound_assign value=%d\n", (int)(g_var_float));
    }

    // Case 234: float_global_inc_dec
    {
        g_var_float = 1.5f;
        ++g_var_float;
        --g_var_float;
        check(g_var_float == 1.5f, "float_global_inc_dec");
        std::printf("CASE 234: float_global_inc_dec value=%d\n", (int)(g_var_float));
    }

    // Case 235: float_global_read_after_write
    {
        g_var_float = 1.5f;
        g_var_float = 4.5f;
        float read_val = g_var_float;
        check(read_val == 4.5f, "float_global_read_after_write");
        std::printf("CASE 235: float_global_read_after_write value=%d\n", (int)(g_var_float));
    }

    // Case 236: float_global_pass_by_val
    {
        g_var_float = 1.5f;
        take_val_g_float(g_var_float);
        check(g_var_float == 1.5f, "float_global_pass_by_val");
        std::printf("CASE 236: float_global_pass_by_val value=%d\n", (int)(g_var_float));
    }

    // Case 237: float_global_pass_by_ref
    {
        g_var_float = 1.5f;
        take_ref_g_float(g_var_float);
        check(g_var_float == 1.5f, "float_global_pass_by_ref");
        std::printf("CASE 237: float_global_pass_by_ref value=%d\n", (int)(g_var_float));
    }

    // Case 238: float_global_return_by_val
    {
        g_var_float = 1.5f;
        g_var_float = return_g_float();
        check(g_var_float == 4.5f, "float_global_return_by_val");
        std::printf("CASE 238: float_global_return_by_val value=%d\n", (int)(g_var_float));
    }

    // Case 239: float_static_local_assign
    {
        static float s_var_float; s_var_float = 1.5f;
        s_var_float = 4.5f;
        check(s_var_float == 4.5f, "float_static_local_assign");
        std::printf("CASE 239: float_static_local_assign value=%d\n", (int)(s_var_float));
    }

    // Case 240: float_static_local_compound_assign
    {
        static float s_var_float; s_var_float = 1.5f;
        s_var_float += 2.0;
        check(s_var_float == (1.5f) + 2.0, "float_static_local_compound_assign");
        std::printf("CASE 240: float_static_local_compound_assign value=%d\n", (int)(s_var_float));
    }

    // Case 241: float_static_local_inc_dec
    {
        static float s_var_float; s_var_float = 1.5f;
        ++s_var_float;
        --s_var_float;
        check(s_var_float == 1.5f, "float_static_local_inc_dec");
        std::printf("CASE 241: float_static_local_inc_dec value=%d\n", (int)(s_var_float));
    }

    // Case 242: float_static_local_read_after_write
    {
        static float s_var_float; s_var_float = 1.5f;
        s_var_float = 4.5f;
        float read_val = s_var_float;
        check(read_val == 4.5f, "float_static_local_read_after_write");
        std::printf("CASE 242: float_static_local_read_after_write value=%d\n", (int)(s_var_float));
    }

    // Case 243: float_static_local_pass_by_val
    {
        static float s_var_float; s_var_float = 1.5f;
        take_val_g_float(s_var_float);
        check(s_var_float == 1.5f, "float_static_local_pass_by_val");
        std::printf("CASE 243: float_static_local_pass_by_val value=%d\n", (int)(s_var_float));
    }

    // Case 244: float_static_local_pass_by_ref
    {
        static float s_var_float; s_var_float = 1.5f;
        take_ref_g_float(s_var_float);
        check(s_var_float == 1.5f, "float_static_local_pass_by_ref");
        std::printf("CASE 244: float_static_local_pass_by_ref value=%d\n", (int)(s_var_float));
    }

    // Case 245: float_static_local_return_by_val
    {
        static float s_var_float; s_var_float = 1.5f;
        s_var_float = return_g_float();
        check(s_var_float == 4.5f, "float_static_local_return_by_val");
        std::printf("CASE 245: float_static_local_return_by_val value=%d\n", (int)(s_var_float));
    }

    // Case 246: float_struct_member_assign
    {
        Holder_g_float h; h.m = 1.5f;
        h.m = 4.5f;
        check(h.m == 4.5f, "float_struct_member_assign");
        std::printf("CASE 246: float_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 247: float_struct_member_compound_assign
    {
        Holder_g_float h; h.m = 1.5f;
        h.m += 2.0;
        check(h.m == (1.5f) + 2.0, "float_struct_member_compound_assign");
        std::printf("CASE 247: float_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 248: float_struct_member_inc_dec
    {
        Holder_g_float h; h.m = 1.5f;
        ++h.m;
        --h.m;
        check(h.m == 1.5f, "float_struct_member_inc_dec");
        std::printf("CASE 248: float_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 249: float_struct_member_read_after_write
    {
        Holder_g_float h; h.m = 1.5f;
        h.m = 4.5f;
        float read_val = h.m;
        check(read_val == 4.5f, "float_struct_member_read_after_write");
        std::printf("CASE 249: float_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 250: float_struct_member_pass_by_val
    {
        Holder_g_float h; h.m = 1.5f;
        take_val_g_float(h.m);
        check(h.m == 1.5f, "float_struct_member_pass_by_val");
        std::printf("CASE 250: float_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 251: float_struct_member_pass_by_ref
    {
        Holder_g_float h; h.m = 1.5f;
        take_ref_g_float(h.m);
        check(h.m == 1.5f, "float_struct_member_pass_by_ref");
        std::printf("CASE 251: float_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 252: float_struct_member_return_by_val
    {
        Holder_g_float h; h.m = 1.5f;
        h.m = return_g_float();
        check(h.m == 4.5f, "float_struct_member_return_by_val");
        std::printf("CASE 252: float_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 253: float_array_elem_assign
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        arr[1] = 4.5f;
        check(arr[1] == 4.5f, "float_array_elem_assign");
        std::printf("CASE 253: float_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 254: float_array_elem_compound_assign
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        arr[1] += 2.0;
        check(arr[1] == (1.5f) + 2.0, "float_array_elem_compound_assign");
        std::printf("CASE 254: float_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 255: float_array_elem_inc_dec
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        ++arr[1];
        --arr[1];
        check(arr[1] == 1.5f, "float_array_elem_inc_dec");
        std::printf("CASE 255: float_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 256: float_array_elem_read_after_write
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        arr[1] = 4.5f;
        float read_val = arr[1];
        check(read_val == 4.5f, "float_array_elem_read_after_write");
        std::printf("CASE 256: float_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 257: float_array_elem_pass_by_val
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        take_val_g_float(arr[1]);
        check(arr[1] == 1.5f, "float_array_elem_pass_by_val");
        std::printf("CASE 257: float_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 258: float_array_elem_pass_by_ref
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        take_ref_g_float(arr[1]);
        check(arr[1] == 1.5f, "float_array_elem_pass_by_ref");
        std::printf("CASE 258: float_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 259: float_array_elem_return_by_val
    {
        float arr[3] = { 1.5f, 1.5f, 1.5f };
        arr[1] = return_g_float();
        check(arr[1] == 4.5f, "float_array_elem_return_by_val");
        std::printf("CASE 259: float_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 260: float_via_ref_assign
    {
        float base_val = 1.5f;
        float& ref = base_val;
        ref = 4.5f;
        check(ref == 4.5f, "float_via_ref_assign");
        std::printf("CASE 260: float_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 261: float_via_ref_compound_assign
    {
        float base_val = 1.5f;
        float& ref = base_val;
        ref += 2.0;
        check(ref == (1.5f) + 2.0, "float_via_ref_compound_assign");
        std::printf("CASE 261: float_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 262: float_via_ref_inc_dec
    {
        float base_val = 1.5f;
        float& ref = base_val;
        ++ref;
        --ref;
        check(ref == 1.5f, "float_via_ref_inc_dec");
        std::printf("CASE 262: float_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 263: float_via_ref_read_after_write
    {
        float base_val = 1.5f;
        float& ref = base_val;
        ref = 4.5f;
        float read_val = ref;
        check(read_val == 4.5f, "float_via_ref_read_after_write");
        std::printf("CASE 263: float_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 264: float_via_ref_pass_by_val
    {
        float base_val = 1.5f;
        float& ref = base_val;
        take_val_g_float(ref);
        check(ref == 1.5f, "float_via_ref_pass_by_val");
        std::printf("CASE 264: float_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 265: float_via_ref_pass_by_ref
    {
        float base_val = 1.5f;
        float& ref = base_val;
        take_ref_g_float(ref);
        check(ref == 1.5f, "float_via_ref_pass_by_ref");
        std::printf("CASE 265: float_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 266: float_via_ref_return_by_val
    {
        float base_val = 1.5f;
        float& ref = base_val;
        ref = return_g_float();
        check(ref == 4.5f, "float_via_ref_return_by_val");
        std::printf("CASE 266: float_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 267: float_via_ptr_assign
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        *ptr = 4.5f;
        check(*ptr == 4.5f, "float_via_ptr_assign");
        std::printf("CASE 267: float_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 268: float_via_ptr_compound_assign
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        *ptr += 2.0;
        check(*ptr == (1.5f) + 2.0, "float_via_ptr_compound_assign");
        std::printf("CASE 268: float_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 269: float_via_ptr_inc_dec
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 1.5f, "float_via_ptr_inc_dec");
        std::printf("CASE 269: float_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 270: float_via_ptr_read_after_write
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        *ptr = 4.5f;
        float read_val = *ptr;
        check(read_val == 4.5f, "float_via_ptr_read_after_write");
        std::printf("CASE 270: float_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 271: float_via_ptr_pass_by_val
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        take_val_g_float(*ptr);
        check(*ptr == 1.5f, "float_via_ptr_pass_by_val");
        std::printf("CASE 271: float_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 272: float_via_ptr_pass_by_ref
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        take_ref_g_float(*ptr);
        check(*ptr == 1.5f, "float_via_ptr_pass_by_ref");
        std::printf("CASE 272: float_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 273: float_via_ptr_return_by_val
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        *ptr = return_g_float();
        check(*ptr == 4.5f, "float_via_ptr_return_by_val");
        std::printf("CASE 273: float_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 274: float_via_pptr_assign
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr = 4.5f;
        check(**pptr == 4.5f, "float_via_pptr_assign");
        std::printf("CASE 274: float_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 275: float_via_pptr_compound_assign
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr += 2.0;
        check(**pptr == (1.5f) + 2.0, "float_via_pptr_compound_assign");
        std::printf("CASE 275: float_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 276: float_via_pptr_inc_dec
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 1.5f, "float_via_pptr_inc_dec");
        std::printf("CASE 276: float_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 277: float_via_pptr_read_after_write
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr = 4.5f;
        float read_val = **pptr;
        check(read_val == 4.5f, "float_via_pptr_read_after_write");
        std::printf("CASE 277: float_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 278: float_via_pptr_pass_by_val
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        take_val_g_float(**pptr);
        check(**pptr == 1.5f, "float_via_pptr_pass_by_val");
        std::printf("CASE 278: float_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 279: float_via_pptr_pass_by_ref
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        take_ref_g_float(**pptr);
        check(**pptr == 1.5f, "float_via_pptr_pass_by_ref");
        std::printf("CASE 279: float_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 280: float_via_pptr_return_by_val
    {
        float base_val = 1.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr = return_g_float();
        check(**pptr == 4.5f, "float_via_pptr_return_by_val");
        std::printf("CASE 280: float_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 281: double_local_assign
    {
        double lv = 2.25;
        lv = 6.75;
        check(lv == 6.75, "double_local_assign");
        std::printf("CASE 281: double_local_assign value=%d\n", (int)(lv));
    }

    // Case 282: double_local_compound_assign
    {
        double lv = 2.25;
        lv += 3.0;
        check(lv == (2.25) + 3.0, "double_local_compound_assign");
        std::printf("CASE 282: double_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 283: double_local_inc_dec
    {
        double lv = 2.25;
        ++lv;
        --lv;
        check(lv == 2.25, "double_local_inc_dec");
        std::printf("CASE 283: double_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 284: double_local_read_after_write
    {
        double lv = 2.25;
        lv = 6.75;
        double read_val = lv;
        check(read_val == 6.75, "double_local_read_after_write");
        std::printf("CASE 284: double_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 285: double_local_pass_by_val
    {
        double lv = 2.25;
        take_val_g_double(lv);
        check(lv == 2.25, "double_local_pass_by_val");
        std::printf("CASE 285: double_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 286: double_local_pass_by_ref
    {
        double lv = 2.25;
        take_ref_g_double(lv);
        check(lv == 2.25, "double_local_pass_by_ref");
        std::printf("CASE 286: double_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 287: double_local_return_by_val
    {
        double lv = 2.25;
        lv = return_g_double();
        check(lv == 6.75, "double_local_return_by_val");
        std::printf("CASE 287: double_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 288: double_global_assign
    {
        g_var_double = 2.25;
        g_var_double = 6.75;
        check(g_var_double == 6.75, "double_global_assign");
        std::printf("CASE 288: double_global_assign value=%d\n", (int)(g_var_double));
    }

    // Case 289: double_global_compound_assign
    {
        g_var_double = 2.25;
        g_var_double += 3.0;
        check(g_var_double == (2.25) + 3.0, "double_global_compound_assign");
        std::printf("CASE 289: double_global_compound_assign value=%d\n", (int)(g_var_double));
    }

    // Case 290: double_global_inc_dec
    {
        g_var_double = 2.25;
        ++g_var_double;
        --g_var_double;
        check(g_var_double == 2.25, "double_global_inc_dec");
        std::printf("CASE 290: double_global_inc_dec value=%d\n", (int)(g_var_double));
    }

    // Case 291: double_global_read_after_write
    {
        g_var_double = 2.25;
        g_var_double = 6.75;
        double read_val = g_var_double;
        check(read_val == 6.75, "double_global_read_after_write");
        std::printf("CASE 291: double_global_read_after_write value=%d\n", (int)(g_var_double));
    }

    // Case 292: double_global_pass_by_val
    {
        g_var_double = 2.25;
        take_val_g_double(g_var_double);
        check(g_var_double == 2.25, "double_global_pass_by_val");
        std::printf("CASE 292: double_global_pass_by_val value=%d\n", (int)(g_var_double));
    }

    // Case 293: double_global_pass_by_ref
    {
        g_var_double = 2.25;
        take_ref_g_double(g_var_double);
        check(g_var_double == 2.25, "double_global_pass_by_ref");
        std::printf("CASE 293: double_global_pass_by_ref value=%d\n", (int)(g_var_double));
    }

    // Case 294: double_global_return_by_val
    {
        g_var_double = 2.25;
        g_var_double = return_g_double();
        check(g_var_double == 6.75, "double_global_return_by_val");
        std::printf("CASE 294: double_global_return_by_val value=%d\n", (int)(g_var_double));
    }

    // Case 295: double_static_local_assign
    {
        static double s_var_double; s_var_double = 2.25;
        s_var_double = 6.75;
        check(s_var_double == 6.75, "double_static_local_assign");
        std::printf("CASE 295: double_static_local_assign value=%d\n", (int)(s_var_double));
    }

    // Case 296: double_static_local_compound_assign
    {
        static double s_var_double; s_var_double = 2.25;
        s_var_double += 3.0;
        check(s_var_double == (2.25) + 3.0, "double_static_local_compound_assign");
        std::printf("CASE 296: double_static_local_compound_assign value=%d\n", (int)(s_var_double));
    }

    // Case 297: double_static_local_inc_dec
    {
        static double s_var_double; s_var_double = 2.25;
        ++s_var_double;
        --s_var_double;
        check(s_var_double == 2.25, "double_static_local_inc_dec");
        std::printf("CASE 297: double_static_local_inc_dec value=%d\n", (int)(s_var_double));
    }

    // Case 298: double_static_local_read_after_write
    {
        static double s_var_double; s_var_double = 2.25;
        s_var_double = 6.75;
        double read_val = s_var_double;
        check(read_val == 6.75, "double_static_local_read_after_write");
        std::printf("CASE 298: double_static_local_read_after_write value=%d\n", (int)(s_var_double));
    }

    // Case 299: double_static_local_pass_by_val
    {
        static double s_var_double; s_var_double = 2.25;
        take_val_g_double(s_var_double);
        check(s_var_double == 2.25, "double_static_local_pass_by_val");
        std::printf("CASE 299: double_static_local_pass_by_val value=%d\n", (int)(s_var_double));
    }

    // Case 300: double_static_local_pass_by_ref
    {
        static double s_var_double; s_var_double = 2.25;
        take_ref_g_double(s_var_double);
        check(s_var_double == 2.25, "double_static_local_pass_by_ref");
        std::printf("CASE 300: double_static_local_pass_by_ref value=%d\n", (int)(s_var_double));
    }

    // Case 301: double_static_local_return_by_val
    {
        static double s_var_double; s_var_double = 2.25;
        s_var_double = return_g_double();
        check(s_var_double == 6.75, "double_static_local_return_by_val");
        std::printf("CASE 301: double_static_local_return_by_val value=%d\n", (int)(s_var_double));
    }

    // Case 302: double_struct_member_assign
    {
        Holder_g_double h; h.m = 2.25;
        h.m = 6.75;
        check(h.m == 6.75, "double_struct_member_assign");
        std::printf("CASE 302: double_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 303: double_struct_member_compound_assign
    {
        Holder_g_double h; h.m = 2.25;
        h.m += 3.0;
        check(h.m == (2.25) + 3.0, "double_struct_member_compound_assign");
        std::printf("CASE 303: double_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 304: double_struct_member_inc_dec
    {
        Holder_g_double h; h.m = 2.25;
        ++h.m;
        --h.m;
        check(h.m == 2.25, "double_struct_member_inc_dec");
        std::printf("CASE 304: double_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 305: double_struct_member_read_after_write
    {
        Holder_g_double h; h.m = 2.25;
        h.m = 6.75;
        double read_val = h.m;
        check(read_val == 6.75, "double_struct_member_read_after_write");
        std::printf("CASE 305: double_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 306: double_struct_member_pass_by_val
    {
        Holder_g_double h; h.m = 2.25;
        take_val_g_double(h.m);
        check(h.m == 2.25, "double_struct_member_pass_by_val");
        std::printf("CASE 306: double_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 307: double_struct_member_pass_by_ref
    {
        Holder_g_double h; h.m = 2.25;
        take_ref_g_double(h.m);
        check(h.m == 2.25, "double_struct_member_pass_by_ref");
        std::printf("CASE 307: double_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 308: double_struct_member_return_by_val
    {
        Holder_g_double h; h.m = 2.25;
        h.m = return_g_double();
        check(h.m == 6.75, "double_struct_member_return_by_val");
        std::printf("CASE 308: double_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 309: double_array_elem_assign
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        arr[1] = 6.75;
        check(arr[1] == 6.75, "double_array_elem_assign");
        std::printf("CASE 309: double_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 310: double_array_elem_compound_assign
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        arr[1] += 3.0;
        check(arr[1] == (2.25) + 3.0, "double_array_elem_compound_assign");
        std::printf("CASE 310: double_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 311: double_array_elem_inc_dec
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        ++arr[1];
        --arr[1];
        check(arr[1] == 2.25, "double_array_elem_inc_dec");
        std::printf("CASE 311: double_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 312: double_array_elem_read_after_write
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        arr[1] = 6.75;
        double read_val = arr[1];
        check(read_val == 6.75, "double_array_elem_read_after_write");
        std::printf("CASE 312: double_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 313: double_array_elem_pass_by_val
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        take_val_g_double(arr[1]);
        check(arr[1] == 2.25, "double_array_elem_pass_by_val");
        std::printf("CASE 313: double_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 314: double_array_elem_pass_by_ref
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        take_ref_g_double(arr[1]);
        check(arr[1] == 2.25, "double_array_elem_pass_by_ref");
        std::printf("CASE 314: double_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 315: double_array_elem_return_by_val
    {
        double arr[3] = { 2.25, 2.25, 2.25 };
        arr[1] = return_g_double();
        check(arr[1] == 6.75, "double_array_elem_return_by_val");
        std::printf("CASE 315: double_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 316: double_via_ref_assign
    {
        double base_val = 2.25;
        double& ref = base_val;
        ref = 6.75;
        check(ref == 6.75, "double_via_ref_assign");
        std::printf("CASE 316: double_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 317: double_via_ref_compound_assign
    {
        double base_val = 2.25;
        double& ref = base_val;
        ref += 3.0;
        check(ref == (2.25) + 3.0, "double_via_ref_compound_assign");
        std::printf("CASE 317: double_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 318: double_via_ref_inc_dec
    {
        double base_val = 2.25;
        double& ref = base_val;
        ++ref;
        --ref;
        check(ref == 2.25, "double_via_ref_inc_dec");
        std::printf("CASE 318: double_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 319: double_via_ref_read_after_write
    {
        double base_val = 2.25;
        double& ref = base_val;
        ref = 6.75;
        double read_val = ref;
        check(read_val == 6.75, "double_via_ref_read_after_write");
        std::printf("CASE 319: double_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 320: double_via_ref_pass_by_val
    {
        double base_val = 2.25;
        double& ref = base_val;
        take_val_g_double(ref);
        check(ref == 2.25, "double_via_ref_pass_by_val");
        std::printf("CASE 320: double_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 321: double_via_ref_pass_by_ref
    {
        double base_val = 2.25;
        double& ref = base_val;
        take_ref_g_double(ref);
        check(ref == 2.25, "double_via_ref_pass_by_ref");
        std::printf("CASE 321: double_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 322: double_via_ref_return_by_val
    {
        double base_val = 2.25;
        double& ref = base_val;
        ref = return_g_double();
        check(ref == 6.75, "double_via_ref_return_by_val");
        std::printf("CASE 322: double_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 323: double_via_ptr_assign
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        *ptr = 6.75;
        check(*ptr == 6.75, "double_via_ptr_assign");
        std::printf("CASE 323: double_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 324: double_via_ptr_compound_assign
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        *ptr += 3.0;
        check(*ptr == (2.25) + 3.0, "double_via_ptr_compound_assign");
        std::printf("CASE 324: double_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 325: double_via_ptr_inc_dec
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 2.25, "double_via_ptr_inc_dec");
        std::printf("CASE 325: double_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 326: double_via_ptr_read_after_write
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        *ptr = 6.75;
        double read_val = *ptr;
        check(read_val == 6.75, "double_via_ptr_read_after_write");
        std::printf("CASE 326: double_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 327: double_via_ptr_pass_by_val
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        take_val_g_double(*ptr);
        check(*ptr == 2.25, "double_via_ptr_pass_by_val");
        std::printf("CASE 327: double_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 328: double_via_ptr_pass_by_ref
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        take_ref_g_double(*ptr);
        check(*ptr == 2.25, "double_via_ptr_pass_by_ref");
        std::printf("CASE 328: double_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 329: double_via_ptr_return_by_val
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        *ptr = return_g_double();
        check(*ptr == 6.75, "double_via_ptr_return_by_val");
        std::printf("CASE 329: double_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 330: double_via_pptr_assign
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr = 6.75;
        check(**pptr == 6.75, "double_via_pptr_assign");
        std::printf("CASE 330: double_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 331: double_via_pptr_compound_assign
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr += 3.0;
        check(**pptr == (2.25) + 3.0, "double_via_pptr_compound_assign");
        std::printf("CASE 331: double_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 332: double_via_pptr_inc_dec
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 2.25, "double_via_pptr_inc_dec");
        std::printf("CASE 332: double_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 333: double_via_pptr_read_after_write
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr = 6.75;
        double read_val = **pptr;
        check(read_val == 6.75, "double_via_pptr_read_after_write");
        std::printf("CASE 333: double_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 334: double_via_pptr_pass_by_val
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        take_val_g_double(**pptr);
        check(**pptr == 2.25, "double_via_pptr_pass_by_val");
        std::printf("CASE 334: double_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 335: double_via_pptr_pass_by_ref
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        take_ref_g_double(**pptr);
        check(**pptr == 2.25, "double_via_pptr_pass_by_ref");
        std::printf("CASE 335: double_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 336: double_via_pptr_return_by_val
    {
        double base_val = 2.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr = return_g_double();
        check(**pptr == 6.75, "double_via_pptr_return_by_val");
        std::printf("CASE 336: double_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 337: bool_local_assign
    {
        bool lv = true;
        lv = false;
        check(lv == false, "bool_local_assign");
        std::printf("CASE 337: bool_local_assign value=%d\n", (int)(lv));
    }

    // Case 338: bool_local_compound_assign
    {
        bool lv = true;
        lv = false;
        check(lv == false, "bool_local_compound_assign");
        std::printf("CASE 338: bool_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 339: bool_local_inc_dec
    {
        bool lv = true;
        lv = true;
        check(lv == true, "bool_local_inc_dec");
        std::printf("CASE 339: bool_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 340: bool_local_read_after_write
    {
        bool lv = true;
        lv = false;
        bool read_val = lv;
        check(read_val == false, "bool_local_read_after_write");
        std::printf("CASE 340: bool_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 341: bool_local_pass_by_val
    {
        bool lv = true;
        take_val_g_bool(lv);
        check(lv == true, "bool_local_pass_by_val");
        std::printf("CASE 341: bool_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 342: bool_local_pass_by_ref
    {
        bool lv = true;
        take_ref_g_bool(lv);
        check(lv == true, "bool_local_pass_by_ref");
        std::printf("CASE 342: bool_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 343: bool_local_return_by_val
    {
        bool lv = true;
        lv = return_g_bool();
        check(lv == false, "bool_local_return_by_val");
        std::printf("CASE 343: bool_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 344: bool_global_assign
    {
        g_var_bool = true;
        g_var_bool = false;
        check(g_var_bool == false, "bool_global_assign");
        std::printf("CASE 344: bool_global_assign value=%d\n", (int)(g_var_bool));
    }

    // Case 345: bool_global_compound_assign
    {
        g_var_bool = true;
        g_var_bool = false;
        check(g_var_bool == false, "bool_global_compound_assign");
        std::printf("CASE 345: bool_global_compound_assign value=%d\n", (int)(g_var_bool));
    }

    // Case 346: bool_global_inc_dec
    {
        g_var_bool = true;
        g_var_bool = true;
        check(g_var_bool == true, "bool_global_inc_dec");
        std::printf("CASE 346: bool_global_inc_dec value=%d\n", (int)(g_var_bool));
    }

    // Case 347: bool_global_read_after_write
    {
        g_var_bool = true;
        g_var_bool = false;
        bool read_val = g_var_bool;
        check(read_val == false, "bool_global_read_after_write");
        std::printf("CASE 347: bool_global_read_after_write value=%d\n", (int)(g_var_bool));
    }

    // Case 348: bool_global_pass_by_val
    {
        g_var_bool = true;
        take_val_g_bool(g_var_bool);
        check(g_var_bool == true, "bool_global_pass_by_val");
        std::printf("CASE 348: bool_global_pass_by_val value=%d\n", (int)(g_var_bool));
    }

    // Case 349: bool_global_pass_by_ref
    {
        g_var_bool = true;
        take_ref_g_bool(g_var_bool);
        check(g_var_bool == true, "bool_global_pass_by_ref");
        std::printf("CASE 349: bool_global_pass_by_ref value=%d\n", (int)(g_var_bool));
    }

    // Case 350: bool_global_return_by_val
    {
        g_var_bool = true;
        g_var_bool = return_g_bool();
        check(g_var_bool == false, "bool_global_return_by_val");
        std::printf("CASE 350: bool_global_return_by_val value=%d\n", (int)(g_var_bool));
    }

    // Case 351: bool_static_local_assign
    {
        static bool s_var_bool; s_var_bool = true;
        s_var_bool = false;
        check(s_var_bool == false, "bool_static_local_assign");
        std::printf("CASE 351: bool_static_local_assign value=%d\n", (int)(s_var_bool));
    }

    // Case 352: bool_static_local_compound_assign
    {
        static bool s_var_bool; s_var_bool = true;
        s_var_bool = false;
        check(s_var_bool == false, "bool_static_local_compound_assign");
        std::printf("CASE 352: bool_static_local_compound_assign value=%d\n", (int)(s_var_bool));
    }

    // Case 353: bool_static_local_inc_dec
    {
        static bool s_var_bool; s_var_bool = true;
        s_var_bool = true;
        check(s_var_bool == true, "bool_static_local_inc_dec");
        std::printf("CASE 353: bool_static_local_inc_dec value=%d\n", (int)(s_var_bool));
    }

    // Case 354: bool_static_local_read_after_write
    {
        static bool s_var_bool; s_var_bool = true;
        s_var_bool = false;
        bool read_val = s_var_bool;
        check(read_val == false, "bool_static_local_read_after_write");
        std::printf("CASE 354: bool_static_local_read_after_write value=%d\n", (int)(s_var_bool));
    }

    // Case 355: bool_static_local_pass_by_val
    {
        static bool s_var_bool; s_var_bool = true;
        take_val_g_bool(s_var_bool);
        check(s_var_bool == true, "bool_static_local_pass_by_val");
        std::printf("CASE 355: bool_static_local_pass_by_val value=%d\n", (int)(s_var_bool));
    }

    // Case 356: bool_static_local_pass_by_ref
    {
        static bool s_var_bool; s_var_bool = true;
        take_ref_g_bool(s_var_bool);
        check(s_var_bool == true, "bool_static_local_pass_by_ref");
        std::printf("CASE 356: bool_static_local_pass_by_ref value=%d\n", (int)(s_var_bool));
    }

    // Case 357: bool_static_local_return_by_val
    {
        static bool s_var_bool; s_var_bool = true;
        s_var_bool = return_g_bool();
        check(s_var_bool == false, "bool_static_local_return_by_val");
        std::printf("CASE 357: bool_static_local_return_by_val value=%d\n", (int)(s_var_bool));
    }

    // Case 358: bool_struct_member_assign
    {
        Holder_g_bool h; h.m = true;
        h.m = false;
        check(h.m == false, "bool_struct_member_assign");
        std::printf("CASE 358: bool_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 359: bool_struct_member_compound_assign
    {
        Holder_g_bool h; h.m = true;
        h.m = false;
        check(h.m == false, "bool_struct_member_compound_assign");
        std::printf("CASE 359: bool_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 360: bool_struct_member_inc_dec
    {
        Holder_g_bool h; h.m = true;
        h.m = true;
        check(h.m == true, "bool_struct_member_inc_dec");
        std::printf("CASE 360: bool_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 361: bool_struct_member_read_after_write
    {
        Holder_g_bool h; h.m = true;
        h.m = false;
        bool read_val = h.m;
        check(read_val == false, "bool_struct_member_read_after_write");
        std::printf("CASE 361: bool_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 362: bool_struct_member_pass_by_val
    {
        Holder_g_bool h; h.m = true;
        take_val_g_bool(h.m);
        check(h.m == true, "bool_struct_member_pass_by_val");
        std::printf("CASE 362: bool_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 363: bool_struct_member_pass_by_ref
    {
        Holder_g_bool h; h.m = true;
        take_ref_g_bool(h.m);
        check(h.m == true, "bool_struct_member_pass_by_ref");
        std::printf("CASE 363: bool_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 364: bool_struct_member_return_by_val
    {
        Holder_g_bool h; h.m = true;
        h.m = return_g_bool();
        check(h.m == false, "bool_struct_member_return_by_val");
        std::printf("CASE 364: bool_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 365: bool_array_elem_assign
    {
        bool arr[3] = { true, true, true };
        arr[1] = false;
        check(arr[1] == false, "bool_array_elem_assign");
        std::printf("CASE 365: bool_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 366: bool_array_elem_compound_assign
    {
        bool arr[3] = { true, true, true };
        arr[1] = false;
        check(arr[1] == false, "bool_array_elem_compound_assign");
        std::printf("CASE 366: bool_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 367: bool_array_elem_inc_dec
    {
        bool arr[3] = { true, true, true };
        arr[1] = true;
        check(arr[1] == true, "bool_array_elem_inc_dec");
        std::printf("CASE 367: bool_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 368: bool_array_elem_read_after_write
    {
        bool arr[3] = { true, true, true };
        arr[1] = false;
        bool read_val = arr[1];
        check(read_val == false, "bool_array_elem_read_after_write");
        std::printf("CASE 368: bool_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 369: bool_array_elem_pass_by_val
    {
        bool arr[3] = { true, true, true };
        take_val_g_bool(arr[1]);
        check(arr[1] == true, "bool_array_elem_pass_by_val");
        std::printf("CASE 369: bool_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 370: bool_array_elem_pass_by_ref
    {
        bool arr[3] = { true, true, true };
        take_ref_g_bool(arr[1]);
        check(arr[1] == true, "bool_array_elem_pass_by_ref");
        std::printf("CASE 370: bool_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 371: bool_array_elem_return_by_val
    {
        bool arr[3] = { true, true, true };
        arr[1] = return_g_bool();
        check(arr[1] == false, "bool_array_elem_return_by_val");
        std::printf("CASE 371: bool_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 372: bool_via_ref_assign
    {
        bool base_val = true;
        bool& ref = base_val;
        ref = false;
        check(ref == false, "bool_via_ref_assign");
        std::printf("CASE 372: bool_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 373: bool_via_ref_compound_assign
    {
        bool base_val = true;
        bool& ref = base_val;
        ref = false;
        check(ref == false, "bool_via_ref_compound_assign");
        std::printf("CASE 373: bool_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 374: bool_via_ref_inc_dec
    {
        bool base_val = true;
        bool& ref = base_val;
        ref = true;
        check(ref == true, "bool_via_ref_inc_dec");
        std::printf("CASE 374: bool_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 375: bool_via_ref_read_after_write
    {
        bool base_val = true;
        bool& ref = base_val;
        ref = false;
        bool read_val = ref;
        check(read_val == false, "bool_via_ref_read_after_write");
        std::printf("CASE 375: bool_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 376: bool_via_ref_pass_by_val
    {
        bool base_val = true;
        bool& ref = base_val;
        take_val_g_bool(ref);
        check(ref == true, "bool_via_ref_pass_by_val");
        std::printf("CASE 376: bool_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 377: bool_via_ref_pass_by_ref
    {
        bool base_val = true;
        bool& ref = base_val;
        take_ref_g_bool(ref);
        check(ref == true, "bool_via_ref_pass_by_ref");
        std::printf("CASE 377: bool_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 378: bool_via_ref_return_by_val
    {
        bool base_val = true;
        bool& ref = base_val;
        ref = return_g_bool();
        check(ref == false, "bool_via_ref_return_by_val");
        std::printf("CASE 378: bool_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 379: bool_via_ptr_assign
    {
        bool base_val = true;
        bool* ptr = &base_val;
        *ptr = false;
        check(*ptr == false, "bool_via_ptr_assign");
        std::printf("CASE 379: bool_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 380: bool_via_ptr_compound_assign
    {
        bool base_val = true;
        bool* ptr = &base_val;
        *ptr = false;
        check(*ptr == false, "bool_via_ptr_compound_assign");
        std::printf("CASE 380: bool_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 381: bool_via_ptr_inc_dec
    {
        bool base_val = true;
        bool* ptr = &base_val;
        *ptr = true;
        check(*ptr == true, "bool_via_ptr_inc_dec");
        std::printf("CASE 381: bool_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 382: bool_via_ptr_read_after_write
    {
        bool base_val = true;
        bool* ptr = &base_val;
        *ptr = false;
        bool read_val = *ptr;
        check(read_val == false, "bool_via_ptr_read_after_write");
        std::printf("CASE 382: bool_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 383: bool_via_ptr_pass_by_val
    {
        bool base_val = true;
        bool* ptr = &base_val;
        take_val_g_bool(*ptr);
        check(*ptr == true, "bool_via_ptr_pass_by_val");
        std::printf("CASE 383: bool_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 384: bool_via_ptr_pass_by_ref
    {
        bool base_val = true;
        bool* ptr = &base_val;
        take_ref_g_bool(*ptr);
        check(*ptr == true, "bool_via_ptr_pass_by_ref");
        std::printf("CASE 384: bool_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 385: bool_via_ptr_return_by_val
    {
        bool base_val = true;
        bool* ptr = &base_val;
        *ptr = return_g_bool();
        check(*ptr == false, "bool_via_ptr_return_by_val");
        std::printf("CASE 385: bool_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 386: bool_via_pptr_assign
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = false;
        check(**pptr == false, "bool_via_pptr_assign");
        std::printf("CASE 386: bool_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 387: bool_via_pptr_compound_assign
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = false;
        check(**pptr == false, "bool_via_pptr_compound_assign");
        std::printf("CASE 387: bool_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 388: bool_via_pptr_inc_dec
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = true;
        check(**pptr == true, "bool_via_pptr_inc_dec");
        std::printf("CASE 388: bool_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 389: bool_via_pptr_read_after_write
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = false;
        bool read_val = **pptr;
        check(read_val == false, "bool_via_pptr_read_after_write");
        std::printf("CASE 389: bool_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 390: bool_via_pptr_pass_by_val
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        take_val_g_bool(**pptr);
        check(**pptr == true, "bool_via_pptr_pass_by_val");
        std::printf("CASE 390: bool_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 391: bool_via_pptr_pass_by_ref
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        take_ref_g_bool(**pptr);
        check(**pptr == true, "bool_via_pptr_pass_by_ref");
        std::printf("CASE 391: bool_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 392: bool_via_pptr_return_by_val
    {
        bool base_val = true;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = return_g_bool();
        check(**pptr == false, "bool_via_pptr_return_by_val");
        std::printf("CASE 392: bool_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 393: char_ptr_local_assign
    {
        char* lv = g_char_buf + 0;
        lv = g_char_buf + 2;
        check(lv == g_char_buf + 2, "char_ptr_local_assign");
        std::printf("CASE 393: char_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 394: char_ptr_local_compound_assign
    {
        char* lv = g_char_buf + 0;
        lv += 1;
        check(lv == (g_char_buf + 0) + 1, "char_ptr_local_compound_assign");
        std::printf("CASE 394: char_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 395: char_ptr_local_inc_dec
    {
        char* lv = g_char_buf + 0;
        ++lv;
        --lv;
        check(lv == g_char_buf + 0, "char_ptr_local_inc_dec");
        std::printf("CASE 395: char_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 396: char_ptr_local_read_after_write
    {
        char* lv = g_char_buf + 0;
        lv = g_char_buf + 2;
        char* read_val = lv;
        check(read_val == g_char_buf + 2, "char_ptr_local_read_after_write");
        std::printf("CASE 396: char_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 397: char_ptr_local_pass_by_val
    {
        char* lv = g_char_buf + 0;
        take_val_g_char_ptr(lv);
        check(lv == g_char_buf + 0, "char_ptr_local_pass_by_val");
        std::printf("CASE 397: char_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 398: char_ptr_local_pass_by_ref
    {
        char* lv = g_char_buf + 0;
        take_ref_g_char_ptr(lv);
        check(lv == g_char_buf + 0, "char_ptr_local_pass_by_ref");
        std::printf("CASE 398: char_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 399: char_ptr_local_return_by_val
    {
        char* lv = g_char_buf + 0;
        lv = return_g_char_ptr();
        check(lv == g_char_buf + 2, "char_ptr_local_return_by_val");
        std::printf("CASE 399: char_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 400: char_ptr_global_assign
    {
        g_var_char_ptr = g_char_buf + 0;
        g_var_char_ptr = g_char_buf + 2;
        check(g_var_char_ptr == g_char_buf + 2, "char_ptr_global_assign");
        std::printf("CASE 400: char_ptr_global_assign value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 401: char_ptr_global_compound_assign
    {
        g_var_char_ptr = g_char_buf + 0;
        g_var_char_ptr += 1;
        check(g_var_char_ptr == (g_char_buf + 0) + 1, "char_ptr_global_compound_assign");
        std::printf("CASE 401: char_ptr_global_compound_assign value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 402: char_ptr_global_inc_dec
    {
        g_var_char_ptr = g_char_buf + 0;
        ++g_var_char_ptr;
        --g_var_char_ptr;
        check(g_var_char_ptr == g_char_buf + 0, "char_ptr_global_inc_dec");
        std::printf("CASE 402: char_ptr_global_inc_dec value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 403: char_ptr_global_read_after_write
    {
        g_var_char_ptr = g_char_buf + 0;
        g_var_char_ptr = g_char_buf + 2;
        char* read_val = g_var_char_ptr;
        check(read_val == g_char_buf + 2, "char_ptr_global_read_after_write");
        std::printf("CASE 403: char_ptr_global_read_after_write value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 404: char_ptr_global_pass_by_val
    {
        g_var_char_ptr = g_char_buf + 0;
        take_val_g_char_ptr(g_var_char_ptr);
        check(g_var_char_ptr == g_char_buf + 0, "char_ptr_global_pass_by_val");
        std::printf("CASE 404: char_ptr_global_pass_by_val value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 405: char_ptr_global_pass_by_ref
    {
        g_var_char_ptr = g_char_buf + 0;
        take_ref_g_char_ptr(g_var_char_ptr);
        check(g_var_char_ptr == g_char_buf + 0, "char_ptr_global_pass_by_ref");
        std::printf("CASE 405: char_ptr_global_pass_by_ref value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 406: char_ptr_global_return_by_val
    {
        g_var_char_ptr = g_char_buf + 0;
        g_var_char_ptr = return_g_char_ptr();
        check(g_var_char_ptr == g_char_buf + 2, "char_ptr_global_return_by_val");
        std::printf("CASE 406: char_ptr_global_return_by_val value=%d\n", (g_var_char_ptr != nullptr));
    }

    // Case 407: char_ptr_static_local_assign
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        s_var_char_ptr = g_char_buf + 2;
        check(s_var_char_ptr == g_char_buf + 2, "char_ptr_static_local_assign");
        std::printf("CASE 407: char_ptr_static_local_assign value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 408: char_ptr_static_local_compound_assign
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        s_var_char_ptr += 1;
        check(s_var_char_ptr == (g_char_buf + 0) + 1, "char_ptr_static_local_compound_assign");
        std::printf("CASE 408: char_ptr_static_local_compound_assign value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 409: char_ptr_static_local_inc_dec
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        ++s_var_char_ptr;
        --s_var_char_ptr;
        check(s_var_char_ptr == g_char_buf + 0, "char_ptr_static_local_inc_dec");
        std::printf("CASE 409: char_ptr_static_local_inc_dec value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 410: char_ptr_static_local_read_after_write
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        s_var_char_ptr = g_char_buf + 2;
        char* read_val = s_var_char_ptr;
        check(read_val == g_char_buf + 2, "char_ptr_static_local_read_after_write");
        std::printf("CASE 410: char_ptr_static_local_read_after_write value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 411: char_ptr_static_local_pass_by_val
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        take_val_g_char_ptr(s_var_char_ptr);
        check(s_var_char_ptr == g_char_buf + 0, "char_ptr_static_local_pass_by_val");
        std::printf("CASE 411: char_ptr_static_local_pass_by_val value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 412: char_ptr_static_local_pass_by_ref
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        take_ref_g_char_ptr(s_var_char_ptr);
        check(s_var_char_ptr == g_char_buf + 0, "char_ptr_static_local_pass_by_ref");
        std::printf("CASE 412: char_ptr_static_local_pass_by_ref value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 413: char_ptr_static_local_return_by_val
    {
        static char* s_var_char_ptr; s_var_char_ptr = g_char_buf + 0;
        s_var_char_ptr = return_g_char_ptr();
        check(s_var_char_ptr == g_char_buf + 2, "char_ptr_static_local_return_by_val");
        std::printf("CASE 413: char_ptr_static_local_return_by_val value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 414: char_ptr_struct_member_assign
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        h.m = g_char_buf + 2;
        check(h.m == g_char_buf + 2, "char_ptr_struct_member_assign");
        std::printf("CASE 414: char_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 415: char_ptr_struct_member_compound_assign
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        h.m += 1;
        check(h.m == (g_char_buf + 0) + 1, "char_ptr_struct_member_compound_assign");
        std::printf("CASE 415: char_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 416: char_ptr_struct_member_inc_dec
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        ++h.m;
        --h.m;
        check(h.m == g_char_buf + 0, "char_ptr_struct_member_inc_dec");
        std::printf("CASE 416: char_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 417: char_ptr_struct_member_read_after_write
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        h.m = g_char_buf + 2;
        char* read_val = h.m;
        check(read_val == g_char_buf + 2, "char_ptr_struct_member_read_after_write");
        std::printf("CASE 417: char_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 418: char_ptr_struct_member_pass_by_val
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        take_val_g_char_ptr(h.m);
        check(h.m == g_char_buf + 0, "char_ptr_struct_member_pass_by_val");
        std::printf("CASE 418: char_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 419: char_ptr_struct_member_pass_by_ref
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        take_ref_g_char_ptr(h.m);
        check(h.m == g_char_buf + 0, "char_ptr_struct_member_pass_by_ref");
        std::printf("CASE 419: char_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 420: char_ptr_struct_member_return_by_val
    {
        Holder_g_char_ptr h; h.m = g_char_buf + 0;
        h.m = return_g_char_ptr();
        check(h.m == g_char_buf + 2, "char_ptr_struct_member_return_by_val");
        std::printf("CASE 420: char_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 421: char_ptr_array_elem_assign
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        arr[1] = g_char_buf + 2;
        check(arr[1] == g_char_buf + 2, "char_ptr_array_elem_assign");
        std::printf("CASE 421: char_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 422: char_ptr_array_elem_compound_assign
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        arr[1] += 1;
        check(arr[1] == (g_char_buf + 0) + 1, "char_ptr_array_elem_compound_assign");
        std::printf("CASE 422: char_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 423: char_ptr_array_elem_inc_dec
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        ++arr[1];
        --arr[1];
        check(arr[1] == g_char_buf + 0, "char_ptr_array_elem_inc_dec");
        std::printf("CASE 423: char_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 424: char_ptr_array_elem_read_after_write
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        arr[1] = g_char_buf + 2;
        char* read_val = arr[1];
        check(read_val == g_char_buf + 2, "char_ptr_array_elem_read_after_write");
        std::printf("CASE 424: char_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 425: char_ptr_array_elem_pass_by_val
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        take_val_g_char_ptr(arr[1]);
        check(arr[1] == g_char_buf + 0, "char_ptr_array_elem_pass_by_val");
        std::printf("CASE 425: char_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 426: char_ptr_array_elem_pass_by_ref
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        take_ref_g_char_ptr(arr[1]);
        check(arr[1] == g_char_buf + 0, "char_ptr_array_elem_pass_by_ref");
        std::printf("CASE 426: char_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 427: char_ptr_array_elem_return_by_val
    {
        char* arr[3] = { g_char_buf + 0, g_char_buf + 0, g_char_buf + 0 };
        arr[1] = return_g_char_ptr();
        check(arr[1] == g_char_buf + 2, "char_ptr_array_elem_return_by_val");
        std::printf("CASE 427: char_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 428: char_ptr_via_ref_assign
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        ref = g_char_buf + 2;
        check(ref == g_char_buf + 2, "char_ptr_via_ref_assign");
        std::printf("CASE 428: char_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 429: char_ptr_via_ref_compound_assign
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        ref += 1;
        check(ref == (g_char_buf + 0) + 1, "char_ptr_via_ref_compound_assign");
        std::printf("CASE 429: char_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 430: char_ptr_via_ref_inc_dec
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        ++ref;
        --ref;
        check(ref == g_char_buf + 0, "char_ptr_via_ref_inc_dec");
        std::printf("CASE 430: char_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 431: char_ptr_via_ref_read_after_write
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        ref = g_char_buf + 2;
        char* read_val = ref;
        check(read_val == g_char_buf + 2, "char_ptr_via_ref_read_after_write");
        std::printf("CASE 431: char_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 432: char_ptr_via_ref_pass_by_val
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        take_val_g_char_ptr(ref);
        check(ref == g_char_buf + 0, "char_ptr_via_ref_pass_by_val");
        std::printf("CASE 432: char_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 433: char_ptr_via_ref_pass_by_ref
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        take_ref_g_char_ptr(ref);
        check(ref == g_char_buf + 0, "char_ptr_via_ref_pass_by_ref");
        std::printf("CASE 433: char_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 434: char_ptr_via_ref_return_by_val
    {
        char* base_val = g_char_buf + 0;
        char*& ref = base_val;
        ref = return_g_char_ptr();
        check(ref == g_char_buf + 2, "char_ptr_via_ref_return_by_val");
        std::printf("CASE 434: char_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 435: char_ptr_via_ptr_assign
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        *ptr = g_char_buf + 2;
        check(*ptr == g_char_buf + 2, "char_ptr_via_ptr_assign");
        std::printf("CASE 435: char_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 436: char_ptr_via_ptr_compound_assign
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (g_char_buf + 0) + 1, "char_ptr_via_ptr_compound_assign");
        std::printf("CASE 436: char_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 437: char_ptr_via_ptr_inc_dec
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == g_char_buf + 0, "char_ptr_via_ptr_inc_dec");
        std::printf("CASE 437: char_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 438: char_ptr_via_ptr_read_after_write
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        *ptr = g_char_buf + 2;
        char* read_val = *ptr;
        check(read_val == g_char_buf + 2, "char_ptr_via_ptr_read_after_write");
        std::printf("CASE 438: char_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 439: char_ptr_via_ptr_pass_by_val
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        take_val_g_char_ptr(*ptr);
        check(*ptr == g_char_buf + 0, "char_ptr_via_ptr_pass_by_val");
        std::printf("CASE 439: char_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 440: char_ptr_via_ptr_pass_by_ref
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        take_ref_g_char_ptr(*ptr);
        check(*ptr == g_char_buf + 0, "char_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 440: char_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 441: char_ptr_via_ptr_return_by_val
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        *ptr = return_g_char_ptr();
        check(*ptr == g_char_buf + 2, "char_ptr_via_ptr_return_by_val");
        std::printf("CASE 441: char_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 442: char_ptr_via_pptr_assign
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr = g_char_buf + 2;
        check(**pptr == g_char_buf + 2, "char_ptr_via_pptr_assign");
        std::printf("CASE 442: char_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 443: char_ptr_via_pptr_compound_assign
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (g_char_buf + 0) + 1, "char_ptr_via_pptr_compound_assign");
        std::printf("CASE 443: char_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 444: char_ptr_via_pptr_inc_dec
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == g_char_buf + 0, "char_ptr_via_pptr_inc_dec");
        std::printf("CASE 444: char_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 445: char_ptr_via_pptr_read_after_write
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr = g_char_buf + 2;
        char* read_val = **pptr;
        check(read_val == g_char_buf + 2, "char_ptr_via_pptr_read_after_write");
        std::printf("CASE 445: char_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 446: char_ptr_via_pptr_pass_by_val
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        take_val_g_char_ptr(**pptr);
        check(**pptr == g_char_buf + 0, "char_ptr_via_pptr_pass_by_val");
        std::printf("CASE 446: char_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 447: char_ptr_via_pptr_pass_by_ref
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        take_ref_g_char_ptr(**pptr);
        check(**pptr == g_char_buf + 0, "char_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 447: char_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 448: char_ptr_via_pptr_return_by_val
    {
        char* base_val = g_char_buf + 0;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr = return_g_char_ptr();
        check(**pptr == g_char_buf + 2, "char_ptr_via_pptr_return_by_val");
        std::printf("CASE 448: char_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 449: int_ptr_local_assign
    {
        int* lv = g_int_buf + 0;
        lv = g_int_buf + 2;
        check(lv == g_int_buf + 2, "int_ptr_local_assign");
        std::printf("CASE 449: int_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 450: int_ptr_local_compound_assign
    {
        int* lv = g_int_buf + 0;
        lv += 1;
        check(lv == (g_int_buf + 0) + 1, "int_ptr_local_compound_assign");
        std::printf("CASE 450: int_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 451: int_ptr_local_inc_dec
    {
        int* lv = g_int_buf + 0;
        ++lv;
        --lv;
        check(lv == g_int_buf + 0, "int_ptr_local_inc_dec");
        std::printf("CASE 451: int_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 452: int_ptr_local_read_after_write
    {
        int* lv = g_int_buf + 0;
        lv = g_int_buf + 2;
        int* read_val = lv;
        check(read_val == g_int_buf + 2, "int_ptr_local_read_after_write");
        std::printf("CASE 452: int_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 453: int_ptr_local_pass_by_val
    {
        int* lv = g_int_buf + 0;
        take_val_g_int_ptr(lv);
        check(lv == g_int_buf + 0, "int_ptr_local_pass_by_val");
        std::printf("CASE 453: int_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 454: int_ptr_local_pass_by_ref
    {
        int* lv = g_int_buf + 0;
        take_ref_g_int_ptr(lv);
        check(lv == g_int_buf + 0, "int_ptr_local_pass_by_ref");
        std::printf("CASE 454: int_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 455: int_ptr_local_return_by_val
    {
        int* lv = g_int_buf + 0;
        lv = return_g_int_ptr();
        check(lv == g_int_buf + 2, "int_ptr_local_return_by_val");
        std::printf("CASE 455: int_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 456: int_ptr_global_assign
    {
        g_var_int_ptr = g_int_buf + 0;
        g_var_int_ptr = g_int_buf + 2;
        check(g_var_int_ptr == g_int_buf + 2, "int_ptr_global_assign");
        std::printf("CASE 456: int_ptr_global_assign value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 457: int_ptr_global_compound_assign
    {
        g_var_int_ptr = g_int_buf + 0;
        g_var_int_ptr += 1;
        check(g_var_int_ptr == (g_int_buf + 0) + 1, "int_ptr_global_compound_assign");
        std::printf("CASE 457: int_ptr_global_compound_assign value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 458: int_ptr_global_inc_dec
    {
        g_var_int_ptr = g_int_buf + 0;
        ++g_var_int_ptr;
        --g_var_int_ptr;
        check(g_var_int_ptr == g_int_buf + 0, "int_ptr_global_inc_dec");
        std::printf("CASE 458: int_ptr_global_inc_dec value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 459: int_ptr_global_read_after_write
    {
        g_var_int_ptr = g_int_buf + 0;
        g_var_int_ptr = g_int_buf + 2;
        int* read_val = g_var_int_ptr;
        check(read_val == g_int_buf + 2, "int_ptr_global_read_after_write");
        std::printf("CASE 459: int_ptr_global_read_after_write value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 460: int_ptr_global_pass_by_val
    {
        g_var_int_ptr = g_int_buf + 0;
        take_val_g_int_ptr(g_var_int_ptr);
        check(g_var_int_ptr == g_int_buf + 0, "int_ptr_global_pass_by_val");
        std::printf("CASE 460: int_ptr_global_pass_by_val value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 461: int_ptr_global_pass_by_ref
    {
        g_var_int_ptr = g_int_buf + 0;
        take_ref_g_int_ptr(g_var_int_ptr);
        check(g_var_int_ptr == g_int_buf + 0, "int_ptr_global_pass_by_ref");
        std::printf("CASE 461: int_ptr_global_pass_by_ref value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 462: int_ptr_global_return_by_val
    {
        g_var_int_ptr = g_int_buf + 0;
        g_var_int_ptr = return_g_int_ptr();
        check(g_var_int_ptr == g_int_buf + 2, "int_ptr_global_return_by_val");
        std::printf("CASE 462: int_ptr_global_return_by_val value=%d\n", (g_var_int_ptr != nullptr));
    }

    // Case 463: int_ptr_static_local_assign
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        s_var_int_ptr = g_int_buf + 2;
        check(s_var_int_ptr == g_int_buf + 2, "int_ptr_static_local_assign");
        std::printf("CASE 463: int_ptr_static_local_assign value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 464: int_ptr_static_local_compound_assign
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        s_var_int_ptr += 1;
        check(s_var_int_ptr == (g_int_buf + 0) + 1, "int_ptr_static_local_compound_assign");
        std::printf("CASE 464: int_ptr_static_local_compound_assign value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 465: int_ptr_static_local_inc_dec
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        ++s_var_int_ptr;
        --s_var_int_ptr;
        check(s_var_int_ptr == g_int_buf + 0, "int_ptr_static_local_inc_dec");
        std::printf("CASE 465: int_ptr_static_local_inc_dec value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 466: int_ptr_static_local_read_after_write
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        s_var_int_ptr = g_int_buf + 2;
        int* read_val = s_var_int_ptr;
        check(read_val == g_int_buf + 2, "int_ptr_static_local_read_after_write");
        std::printf("CASE 466: int_ptr_static_local_read_after_write value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 467: int_ptr_static_local_pass_by_val
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        take_val_g_int_ptr(s_var_int_ptr);
        check(s_var_int_ptr == g_int_buf + 0, "int_ptr_static_local_pass_by_val");
        std::printf("CASE 467: int_ptr_static_local_pass_by_val value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 468: int_ptr_static_local_pass_by_ref
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        take_ref_g_int_ptr(s_var_int_ptr);
        check(s_var_int_ptr == g_int_buf + 0, "int_ptr_static_local_pass_by_ref");
        std::printf("CASE 468: int_ptr_static_local_pass_by_ref value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 469: int_ptr_static_local_return_by_val
    {
        static int* s_var_int_ptr; s_var_int_ptr = g_int_buf + 0;
        s_var_int_ptr = return_g_int_ptr();
        check(s_var_int_ptr == g_int_buf + 2, "int_ptr_static_local_return_by_val");
        std::printf("CASE 469: int_ptr_static_local_return_by_val value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 470: int_ptr_struct_member_assign
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        h.m = g_int_buf + 2;
        check(h.m == g_int_buf + 2, "int_ptr_struct_member_assign");
        std::printf("CASE 470: int_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 471: int_ptr_struct_member_compound_assign
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        h.m += 1;
        check(h.m == (g_int_buf + 0) + 1, "int_ptr_struct_member_compound_assign");
        std::printf("CASE 471: int_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 472: int_ptr_struct_member_inc_dec
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        ++h.m;
        --h.m;
        check(h.m == g_int_buf + 0, "int_ptr_struct_member_inc_dec");
        std::printf("CASE 472: int_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 473: int_ptr_struct_member_read_after_write
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        h.m = g_int_buf + 2;
        int* read_val = h.m;
        check(read_val == g_int_buf + 2, "int_ptr_struct_member_read_after_write");
        std::printf("CASE 473: int_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 474: int_ptr_struct_member_pass_by_val
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        take_val_g_int_ptr(h.m);
        check(h.m == g_int_buf + 0, "int_ptr_struct_member_pass_by_val");
        std::printf("CASE 474: int_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 475: int_ptr_struct_member_pass_by_ref
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        take_ref_g_int_ptr(h.m);
        check(h.m == g_int_buf + 0, "int_ptr_struct_member_pass_by_ref");
        std::printf("CASE 475: int_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 476: int_ptr_struct_member_return_by_val
    {
        Holder_g_int_ptr h; h.m = g_int_buf + 0;
        h.m = return_g_int_ptr();
        check(h.m == g_int_buf + 2, "int_ptr_struct_member_return_by_val");
        std::printf("CASE 476: int_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 477: int_ptr_array_elem_assign
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        arr[1] = g_int_buf + 2;
        check(arr[1] == g_int_buf + 2, "int_ptr_array_elem_assign");
        std::printf("CASE 477: int_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 478: int_ptr_array_elem_compound_assign
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        arr[1] += 1;
        check(arr[1] == (g_int_buf + 0) + 1, "int_ptr_array_elem_compound_assign");
        std::printf("CASE 478: int_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 479: int_ptr_array_elem_inc_dec
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        ++arr[1];
        --arr[1];
        check(arr[1] == g_int_buf + 0, "int_ptr_array_elem_inc_dec");
        std::printf("CASE 479: int_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 480: int_ptr_array_elem_read_after_write
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        arr[1] = g_int_buf + 2;
        int* read_val = arr[1];
        check(read_val == g_int_buf + 2, "int_ptr_array_elem_read_after_write");
        std::printf("CASE 480: int_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 481: int_ptr_array_elem_pass_by_val
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        take_val_g_int_ptr(arr[1]);
        check(arr[1] == g_int_buf + 0, "int_ptr_array_elem_pass_by_val");
        std::printf("CASE 481: int_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 482: int_ptr_array_elem_pass_by_ref
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        take_ref_g_int_ptr(arr[1]);
        check(arr[1] == g_int_buf + 0, "int_ptr_array_elem_pass_by_ref");
        std::printf("CASE 482: int_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 483: int_ptr_array_elem_return_by_val
    {
        int* arr[3] = { g_int_buf + 0, g_int_buf + 0, g_int_buf + 0 };
        arr[1] = return_g_int_ptr();
        check(arr[1] == g_int_buf + 2, "int_ptr_array_elem_return_by_val");
        std::printf("CASE 483: int_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 484: int_ptr_via_ref_assign
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        ref = g_int_buf + 2;
        check(ref == g_int_buf + 2, "int_ptr_via_ref_assign");
        std::printf("CASE 484: int_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 485: int_ptr_via_ref_compound_assign
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        ref += 1;
        check(ref == (g_int_buf + 0) + 1, "int_ptr_via_ref_compound_assign");
        std::printf("CASE 485: int_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 486: int_ptr_via_ref_inc_dec
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        ++ref;
        --ref;
        check(ref == g_int_buf + 0, "int_ptr_via_ref_inc_dec");
        std::printf("CASE 486: int_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 487: int_ptr_via_ref_read_after_write
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        ref = g_int_buf + 2;
        int* read_val = ref;
        check(read_val == g_int_buf + 2, "int_ptr_via_ref_read_after_write");
        std::printf("CASE 487: int_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 488: int_ptr_via_ref_pass_by_val
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        take_val_g_int_ptr(ref);
        check(ref == g_int_buf + 0, "int_ptr_via_ref_pass_by_val");
        std::printf("CASE 488: int_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 489: int_ptr_via_ref_pass_by_ref
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        take_ref_g_int_ptr(ref);
        check(ref == g_int_buf + 0, "int_ptr_via_ref_pass_by_ref");
        std::printf("CASE 489: int_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 490: int_ptr_via_ref_return_by_val
    {
        int* base_val = g_int_buf + 0;
        int*& ref = base_val;
        ref = return_g_int_ptr();
        check(ref == g_int_buf + 2, "int_ptr_via_ref_return_by_val");
        std::printf("CASE 490: int_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 491: int_ptr_via_ptr_assign
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        *ptr = g_int_buf + 2;
        check(*ptr == g_int_buf + 2, "int_ptr_via_ptr_assign");
        std::printf("CASE 491: int_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 492: int_ptr_via_ptr_compound_assign
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (g_int_buf + 0) + 1, "int_ptr_via_ptr_compound_assign");
        std::printf("CASE 492: int_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 493: int_ptr_via_ptr_inc_dec
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == g_int_buf + 0, "int_ptr_via_ptr_inc_dec");
        std::printf("CASE 493: int_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 494: int_ptr_via_ptr_read_after_write
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        *ptr = g_int_buf + 2;
        int* read_val = *ptr;
        check(read_val == g_int_buf + 2, "int_ptr_via_ptr_read_after_write");
        std::printf("CASE 494: int_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 495: int_ptr_via_ptr_pass_by_val
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        take_val_g_int_ptr(*ptr);
        check(*ptr == g_int_buf + 0, "int_ptr_via_ptr_pass_by_val");
        std::printf("CASE 495: int_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 496: int_ptr_via_ptr_pass_by_ref
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        take_ref_g_int_ptr(*ptr);
        check(*ptr == g_int_buf + 0, "int_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 496: int_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 497: int_ptr_via_ptr_return_by_val
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        *ptr = return_g_int_ptr();
        check(*ptr == g_int_buf + 2, "int_ptr_via_ptr_return_by_val");
        std::printf("CASE 497: int_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 498: int_ptr_via_pptr_assign
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr = g_int_buf + 2;
        check(**pptr == g_int_buf + 2, "int_ptr_via_pptr_assign");
        std::printf("CASE 498: int_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 499: int_ptr_via_pptr_compound_assign
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (g_int_buf + 0) + 1, "int_ptr_via_pptr_compound_assign");
        std::printf("CASE 499: int_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 500: int_ptr_via_pptr_inc_dec
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == g_int_buf + 0, "int_ptr_via_pptr_inc_dec");
        std::printf("CASE 500: int_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 501: int_ptr_via_pptr_read_after_write
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr = g_int_buf + 2;
        int* read_val = **pptr;
        check(read_val == g_int_buf + 2, "int_ptr_via_pptr_read_after_write");
        std::printf("CASE 501: int_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 502: int_ptr_via_pptr_pass_by_val
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        take_val_g_int_ptr(**pptr);
        check(**pptr == g_int_buf + 0, "int_ptr_via_pptr_pass_by_val");
        std::printf("CASE 502: int_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 503: int_ptr_via_pptr_pass_by_ref
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        take_ref_g_int_ptr(**pptr);
        check(**pptr == g_int_buf + 0, "int_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 503: int_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 504: int_ptr_via_pptr_return_by_val
    {
        int* base_val = g_int_buf + 0;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr = return_g_int_ptr();
        check(**pptr == g_int_buf + 2, "int_ptr_via_pptr_return_by_val");
        std::printf("CASE 504: int_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 505: double_ptr_local_assign
    {
        double* lv = g_double_buf + 0;
        lv = g_double_buf + 2;
        check(lv == g_double_buf + 2, "double_ptr_local_assign");
        std::printf("CASE 505: double_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 506: double_ptr_local_compound_assign
    {
        double* lv = g_double_buf + 0;
        lv += 1;
        check(lv == (g_double_buf + 0) + 1, "double_ptr_local_compound_assign");
        std::printf("CASE 506: double_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 507: double_ptr_local_inc_dec
    {
        double* lv = g_double_buf + 0;
        ++lv;
        --lv;
        check(lv == g_double_buf + 0, "double_ptr_local_inc_dec");
        std::printf("CASE 507: double_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 508: double_ptr_local_read_after_write
    {
        double* lv = g_double_buf + 0;
        lv = g_double_buf + 2;
        double* read_val = lv;
        check(read_val == g_double_buf + 2, "double_ptr_local_read_after_write");
        std::printf("CASE 508: double_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 509: double_ptr_local_pass_by_val
    {
        double* lv = g_double_buf + 0;
        take_val_g_double_ptr(lv);
        check(lv == g_double_buf + 0, "double_ptr_local_pass_by_val");
        std::printf("CASE 509: double_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 510: double_ptr_local_pass_by_ref
    {
        double* lv = g_double_buf + 0;
        take_ref_g_double_ptr(lv);
        check(lv == g_double_buf + 0, "double_ptr_local_pass_by_ref");
        std::printf("CASE 510: double_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 511: double_ptr_local_return_by_val
    {
        double* lv = g_double_buf + 0;
        lv = return_g_double_ptr();
        check(lv == g_double_buf + 2, "double_ptr_local_return_by_val");
        std::printf("CASE 511: double_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 512: double_ptr_global_assign
    {
        g_var_double_ptr = g_double_buf + 0;
        g_var_double_ptr = g_double_buf + 2;
        check(g_var_double_ptr == g_double_buf + 2, "double_ptr_global_assign");
        std::printf("CASE 512: double_ptr_global_assign value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 513: double_ptr_global_compound_assign
    {
        g_var_double_ptr = g_double_buf + 0;
        g_var_double_ptr += 1;
        check(g_var_double_ptr == (g_double_buf + 0) + 1, "double_ptr_global_compound_assign");
        std::printf("CASE 513: double_ptr_global_compound_assign value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 514: double_ptr_global_inc_dec
    {
        g_var_double_ptr = g_double_buf + 0;
        ++g_var_double_ptr;
        --g_var_double_ptr;
        check(g_var_double_ptr == g_double_buf + 0, "double_ptr_global_inc_dec");
        std::printf("CASE 514: double_ptr_global_inc_dec value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 515: double_ptr_global_read_after_write
    {
        g_var_double_ptr = g_double_buf + 0;
        g_var_double_ptr = g_double_buf + 2;
        double* read_val = g_var_double_ptr;
        check(read_val == g_double_buf + 2, "double_ptr_global_read_after_write");
        std::printf("CASE 515: double_ptr_global_read_after_write value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 516: double_ptr_global_pass_by_val
    {
        g_var_double_ptr = g_double_buf + 0;
        take_val_g_double_ptr(g_var_double_ptr);
        check(g_var_double_ptr == g_double_buf + 0, "double_ptr_global_pass_by_val");
        std::printf("CASE 516: double_ptr_global_pass_by_val value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 517: double_ptr_global_pass_by_ref
    {
        g_var_double_ptr = g_double_buf + 0;
        take_ref_g_double_ptr(g_var_double_ptr);
        check(g_var_double_ptr == g_double_buf + 0, "double_ptr_global_pass_by_ref");
        std::printf("CASE 517: double_ptr_global_pass_by_ref value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 518: double_ptr_global_return_by_val
    {
        g_var_double_ptr = g_double_buf + 0;
        g_var_double_ptr = return_g_double_ptr();
        check(g_var_double_ptr == g_double_buf + 2, "double_ptr_global_return_by_val");
        std::printf("CASE 518: double_ptr_global_return_by_val value=%d\n", (g_var_double_ptr != nullptr));
    }

    // Case 519: double_ptr_static_local_assign
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        s_var_double_ptr = g_double_buf + 2;
        check(s_var_double_ptr == g_double_buf + 2, "double_ptr_static_local_assign");
        std::printf("CASE 519: double_ptr_static_local_assign value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 520: double_ptr_static_local_compound_assign
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        s_var_double_ptr += 1;
        check(s_var_double_ptr == (g_double_buf + 0) + 1, "double_ptr_static_local_compound_assign");
        std::printf("CASE 520: double_ptr_static_local_compound_assign value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 521: double_ptr_static_local_inc_dec
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        ++s_var_double_ptr;
        --s_var_double_ptr;
        check(s_var_double_ptr == g_double_buf + 0, "double_ptr_static_local_inc_dec");
        std::printf("CASE 521: double_ptr_static_local_inc_dec value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 522: double_ptr_static_local_read_after_write
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        s_var_double_ptr = g_double_buf + 2;
        double* read_val = s_var_double_ptr;
        check(read_val == g_double_buf + 2, "double_ptr_static_local_read_after_write");
        std::printf("CASE 522: double_ptr_static_local_read_after_write value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 523: double_ptr_static_local_pass_by_val
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        take_val_g_double_ptr(s_var_double_ptr);
        check(s_var_double_ptr == g_double_buf + 0, "double_ptr_static_local_pass_by_val");
        std::printf("CASE 523: double_ptr_static_local_pass_by_val value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 524: double_ptr_static_local_pass_by_ref
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        take_ref_g_double_ptr(s_var_double_ptr);
        check(s_var_double_ptr == g_double_buf + 0, "double_ptr_static_local_pass_by_ref");
        std::printf("CASE 524: double_ptr_static_local_pass_by_ref value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 525: double_ptr_static_local_return_by_val
    {
        static double* s_var_double_ptr; s_var_double_ptr = g_double_buf + 0;
        s_var_double_ptr = return_g_double_ptr();
        check(s_var_double_ptr == g_double_buf + 2, "double_ptr_static_local_return_by_val");
        std::printf("CASE 525: double_ptr_static_local_return_by_val value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 526: double_ptr_struct_member_assign
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        h.m = g_double_buf + 2;
        check(h.m == g_double_buf + 2, "double_ptr_struct_member_assign");
        std::printf("CASE 526: double_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 527: double_ptr_struct_member_compound_assign
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        h.m += 1;
        check(h.m == (g_double_buf + 0) + 1, "double_ptr_struct_member_compound_assign");
        std::printf("CASE 527: double_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 528: double_ptr_struct_member_inc_dec
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        ++h.m;
        --h.m;
        check(h.m == g_double_buf + 0, "double_ptr_struct_member_inc_dec");
        std::printf("CASE 528: double_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 529: double_ptr_struct_member_read_after_write
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        h.m = g_double_buf + 2;
        double* read_val = h.m;
        check(read_val == g_double_buf + 2, "double_ptr_struct_member_read_after_write");
        std::printf("CASE 529: double_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 530: double_ptr_struct_member_pass_by_val
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        take_val_g_double_ptr(h.m);
        check(h.m == g_double_buf + 0, "double_ptr_struct_member_pass_by_val");
        std::printf("CASE 530: double_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 531: double_ptr_struct_member_pass_by_ref
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        take_ref_g_double_ptr(h.m);
        check(h.m == g_double_buf + 0, "double_ptr_struct_member_pass_by_ref");
        std::printf("CASE 531: double_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 532: double_ptr_struct_member_return_by_val
    {
        Holder_g_double_ptr h; h.m = g_double_buf + 0;
        h.m = return_g_double_ptr();
        check(h.m == g_double_buf + 2, "double_ptr_struct_member_return_by_val");
        std::printf("CASE 532: double_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 533: double_ptr_array_elem_assign
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        arr[1] = g_double_buf + 2;
        check(arr[1] == g_double_buf + 2, "double_ptr_array_elem_assign");
        std::printf("CASE 533: double_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 534: double_ptr_array_elem_compound_assign
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        arr[1] += 1;
        check(arr[1] == (g_double_buf + 0) + 1, "double_ptr_array_elem_compound_assign");
        std::printf("CASE 534: double_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 535: double_ptr_array_elem_inc_dec
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        ++arr[1];
        --arr[1];
        check(arr[1] == g_double_buf + 0, "double_ptr_array_elem_inc_dec");
        std::printf("CASE 535: double_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 536: double_ptr_array_elem_read_after_write
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        arr[1] = g_double_buf + 2;
        double* read_val = arr[1];
        check(read_val == g_double_buf + 2, "double_ptr_array_elem_read_after_write");
        std::printf("CASE 536: double_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 537: double_ptr_array_elem_pass_by_val
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        take_val_g_double_ptr(arr[1]);
        check(arr[1] == g_double_buf + 0, "double_ptr_array_elem_pass_by_val");
        std::printf("CASE 537: double_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 538: double_ptr_array_elem_pass_by_ref
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        take_ref_g_double_ptr(arr[1]);
        check(arr[1] == g_double_buf + 0, "double_ptr_array_elem_pass_by_ref");
        std::printf("CASE 538: double_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 539: double_ptr_array_elem_return_by_val
    {
        double* arr[3] = { g_double_buf + 0, g_double_buf + 0, g_double_buf + 0 };
        arr[1] = return_g_double_ptr();
        check(arr[1] == g_double_buf + 2, "double_ptr_array_elem_return_by_val");
        std::printf("CASE 539: double_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 540: double_ptr_via_ref_assign
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        ref = g_double_buf + 2;
        check(ref == g_double_buf + 2, "double_ptr_via_ref_assign");
        std::printf("CASE 540: double_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 541: double_ptr_via_ref_compound_assign
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        ref += 1;
        check(ref == (g_double_buf + 0) + 1, "double_ptr_via_ref_compound_assign");
        std::printf("CASE 541: double_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 542: double_ptr_via_ref_inc_dec
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        ++ref;
        --ref;
        check(ref == g_double_buf + 0, "double_ptr_via_ref_inc_dec");
        std::printf("CASE 542: double_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 543: double_ptr_via_ref_read_after_write
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        ref = g_double_buf + 2;
        double* read_val = ref;
        check(read_val == g_double_buf + 2, "double_ptr_via_ref_read_after_write");
        std::printf("CASE 543: double_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 544: double_ptr_via_ref_pass_by_val
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        take_val_g_double_ptr(ref);
        check(ref == g_double_buf + 0, "double_ptr_via_ref_pass_by_val");
        std::printf("CASE 544: double_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 545: double_ptr_via_ref_pass_by_ref
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        take_ref_g_double_ptr(ref);
        check(ref == g_double_buf + 0, "double_ptr_via_ref_pass_by_ref");
        std::printf("CASE 545: double_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 546: double_ptr_via_ref_return_by_val
    {
        double* base_val = g_double_buf + 0;
        double*& ref = base_val;
        ref = return_g_double_ptr();
        check(ref == g_double_buf + 2, "double_ptr_via_ref_return_by_val");
        std::printf("CASE 546: double_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 547: double_ptr_via_ptr_assign
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        *ptr = g_double_buf + 2;
        check(*ptr == g_double_buf + 2, "double_ptr_via_ptr_assign");
        std::printf("CASE 547: double_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 548: double_ptr_via_ptr_compound_assign
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (g_double_buf + 0) + 1, "double_ptr_via_ptr_compound_assign");
        std::printf("CASE 548: double_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 549: double_ptr_via_ptr_inc_dec
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == g_double_buf + 0, "double_ptr_via_ptr_inc_dec");
        std::printf("CASE 549: double_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 550: double_ptr_via_ptr_read_after_write
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        *ptr = g_double_buf + 2;
        double* read_val = *ptr;
        check(read_val == g_double_buf + 2, "double_ptr_via_ptr_read_after_write");
        std::printf("CASE 550: double_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 551: double_ptr_via_ptr_pass_by_val
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        take_val_g_double_ptr(*ptr);
        check(*ptr == g_double_buf + 0, "double_ptr_via_ptr_pass_by_val");
        std::printf("CASE 551: double_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 552: double_ptr_via_ptr_pass_by_ref
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        take_ref_g_double_ptr(*ptr);
        check(*ptr == g_double_buf + 0, "double_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 552: double_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 553: double_ptr_via_ptr_return_by_val
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        *ptr = return_g_double_ptr();
        check(*ptr == g_double_buf + 2, "double_ptr_via_ptr_return_by_val");
        std::printf("CASE 553: double_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 554: double_ptr_via_pptr_assign
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr = g_double_buf + 2;
        check(**pptr == g_double_buf + 2, "double_ptr_via_pptr_assign");
        std::printf("CASE 554: double_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 555: double_ptr_via_pptr_compound_assign
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (g_double_buf + 0) + 1, "double_ptr_via_pptr_compound_assign");
        std::printf("CASE 555: double_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 556: double_ptr_via_pptr_inc_dec
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == g_double_buf + 0, "double_ptr_via_pptr_inc_dec");
        std::printf("CASE 556: double_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 557: double_ptr_via_pptr_read_after_write
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr = g_double_buf + 2;
        double* read_val = **pptr;
        check(read_val == g_double_buf + 2, "double_ptr_via_pptr_read_after_write");
        std::printf("CASE 557: double_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 558: double_ptr_via_pptr_pass_by_val
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        take_val_g_double_ptr(**pptr);
        check(**pptr == g_double_buf + 0, "double_ptr_via_pptr_pass_by_val");
        std::printf("CASE 558: double_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 559: double_ptr_via_pptr_pass_by_ref
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        take_ref_g_double_ptr(**pptr);
        check(**pptr == g_double_buf + 0, "double_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 559: double_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 560: double_ptr_via_pptr_return_by_val
    {
        double* base_val = g_double_buf + 0;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr = return_g_double_ptr();
        check(**pptr == g_double_buf + 2, "double_ptr_via_pptr_return_by_val");
        std::printf("CASE 560: double_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 561: struct_ptr_local_assign
    {
        S* lv = g_struct_buf + 0;
        lv = g_struct_buf + 2;
        check(lv == g_struct_buf + 2, "struct_ptr_local_assign");
        std::printf("CASE 561: struct_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 562: struct_ptr_local_compound_assign
    {
        S* lv = g_struct_buf + 0;
        lv += 1;
        check(lv == (g_struct_buf + 0) + 1, "struct_ptr_local_compound_assign");
        std::printf("CASE 562: struct_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 563: struct_ptr_local_inc_dec
    {
        S* lv = g_struct_buf + 0;
        ++lv;
        --lv;
        check(lv == g_struct_buf + 0, "struct_ptr_local_inc_dec");
        std::printf("CASE 563: struct_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 564: struct_ptr_local_read_after_write
    {
        S* lv = g_struct_buf + 0;
        lv = g_struct_buf + 2;
        S* read_val = lv;
        check(read_val == g_struct_buf + 2, "struct_ptr_local_read_after_write");
        std::printf("CASE 564: struct_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 565: struct_ptr_local_pass_by_val
    {
        S* lv = g_struct_buf + 0;
        take_val_g_struct_ptr(lv);
        check(lv == g_struct_buf + 0, "struct_ptr_local_pass_by_val");
        std::printf("CASE 565: struct_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 566: struct_ptr_local_pass_by_ref
    {
        S* lv = g_struct_buf + 0;
        take_ref_g_struct_ptr(lv);
        check(lv == g_struct_buf + 0, "struct_ptr_local_pass_by_ref");
        std::printf("CASE 566: struct_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 567: struct_ptr_local_return_by_val
    {
        S* lv = g_struct_buf + 0;
        lv = return_g_struct_ptr();
        check(lv == g_struct_buf + 2, "struct_ptr_local_return_by_val");
        std::printf("CASE 567: struct_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 568: struct_ptr_global_assign
    {
        g_var_struct_ptr = g_struct_buf + 0;
        g_var_struct_ptr = g_struct_buf + 2;
        check(g_var_struct_ptr == g_struct_buf + 2, "struct_ptr_global_assign");
        std::printf("CASE 568: struct_ptr_global_assign value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 569: struct_ptr_global_compound_assign
    {
        g_var_struct_ptr = g_struct_buf + 0;
        g_var_struct_ptr += 1;
        check(g_var_struct_ptr == (g_struct_buf + 0) + 1, "struct_ptr_global_compound_assign");
        std::printf("CASE 569: struct_ptr_global_compound_assign value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 570: struct_ptr_global_inc_dec
    {
        g_var_struct_ptr = g_struct_buf + 0;
        ++g_var_struct_ptr;
        --g_var_struct_ptr;
        check(g_var_struct_ptr == g_struct_buf + 0, "struct_ptr_global_inc_dec");
        std::printf("CASE 570: struct_ptr_global_inc_dec value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 571: struct_ptr_global_read_after_write
    {
        g_var_struct_ptr = g_struct_buf + 0;
        g_var_struct_ptr = g_struct_buf + 2;
        S* read_val = g_var_struct_ptr;
        check(read_val == g_struct_buf + 2, "struct_ptr_global_read_after_write");
        std::printf("CASE 571: struct_ptr_global_read_after_write value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 572: struct_ptr_global_pass_by_val
    {
        g_var_struct_ptr = g_struct_buf + 0;
        take_val_g_struct_ptr(g_var_struct_ptr);
        check(g_var_struct_ptr == g_struct_buf + 0, "struct_ptr_global_pass_by_val");
        std::printf("CASE 572: struct_ptr_global_pass_by_val value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 573: struct_ptr_global_pass_by_ref
    {
        g_var_struct_ptr = g_struct_buf + 0;
        take_ref_g_struct_ptr(g_var_struct_ptr);
        check(g_var_struct_ptr == g_struct_buf + 0, "struct_ptr_global_pass_by_ref");
        std::printf("CASE 573: struct_ptr_global_pass_by_ref value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 574: struct_ptr_global_return_by_val
    {
        g_var_struct_ptr = g_struct_buf + 0;
        g_var_struct_ptr = return_g_struct_ptr();
        check(g_var_struct_ptr == g_struct_buf + 2, "struct_ptr_global_return_by_val");
        std::printf("CASE 574: struct_ptr_global_return_by_val value=%d\n", (g_var_struct_ptr != nullptr));
    }

    // Case 575: struct_ptr_static_local_assign
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        s_var_struct_ptr = g_struct_buf + 2;
        check(s_var_struct_ptr == g_struct_buf + 2, "struct_ptr_static_local_assign");
        std::printf("CASE 575: struct_ptr_static_local_assign value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 576: struct_ptr_static_local_compound_assign
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        s_var_struct_ptr += 1;
        check(s_var_struct_ptr == (g_struct_buf + 0) + 1, "struct_ptr_static_local_compound_assign");
        std::printf("CASE 576: struct_ptr_static_local_compound_assign value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 577: struct_ptr_static_local_inc_dec
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        ++s_var_struct_ptr;
        --s_var_struct_ptr;
        check(s_var_struct_ptr == g_struct_buf + 0, "struct_ptr_static_local_inc_dec");
        std::printf("CASE 577: struct_ptr_static_local_inc_dec value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 578: struct_ptr_static_local_read_after_write
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        s_var_struct_ptr = g_struct_buf + 2;
        S* read_val = s_var_struct_ptr;
        check(read_val == g_struct_buf + 2, "struct_ptr_static_local_read_after_write");
        std::printf("CASE 578: struct_ptr_static_local_read_after_write value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 579: struct_ptr_static_local_pass_by_val
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        take_val_g_struct_ptr(s_var_struct_ptr);
        check(s_var_struct_ptr == g_struct_buf + 0, "struct_ptr_static_local_pass_by_val");
        std::printf("CASE 579: struct_ptr_static_local_pass_by_val value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 580: struct_ptr_static_local_pass_by_ref
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        take_ref_g_struct_ptr(s_var_struct_ptr);
        check(s_var_struct_ptr == g_struct_buf + 0, "struct_ptr_static_local_pass_by_ref");
        std::printf("CASE 580: struct_ptr_static_local_pass_by_ref value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 581: struct_ptr_static_local_return_by_val
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = g_struct_buf + 0;
        s_var_struct_ptr = return_g_struct_ptr();
        check(s_var_struct_ptr == g_struct_buf + 2, "struct_ptr_static_local_return_by_val");
        std::printf("CASE 581: struct_ptr_static_local_return_by_val value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 582: struct_ptr_struct_member_assign
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        h.m = g_struct_buf + 2;
        check(h.m == g_struct_buf + 2, "struct_ptr_struct_member_assign");
        std::printf("CASE 582: struct_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 583: struct_ptr_struct_member_compound_assign
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        h.m += 1;
        check(h.m == (g_struct_buf + 0) + 1, "struct_ptr_struct_member_compound_assign");
        std::printf("CASE 583: struct_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 584: struct_ptr_struct_member_inc_dec
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        ++h.m;
        --h.m;
        check(h.m == g_struct_buf + 0, "struct_ptr_struct_member_inc_dec");
        std::printf("CASE 584: struct_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 585: struct_ptr_struct_member_read_after_write
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        h.m = g_struct_buf + 2;
        S* read_val = h.m;
        check(read_val == g_struct_buf + 2, "struct_ptr_struct_member_read_after_write");
        std::printf("CASE 585: struct_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 586: struct_ptr_struct_member_pass_by_val
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        take_val_g_struct_ptr(h.m);
        check(h.m == g_struct_buf + 0, "struct_ptr_struct_member_pass_by_val");
        std::printf("CASE 586: struct_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 587: struct_ptr_struct_member_pass_by_ref
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        take_ref_g_struct_ptr(h.m);
        check(h.m == g_struct_buf + 0, "struct_ptr_struct_member_pass_by_ref");
        std::printf("CASE 587: struct_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 588: struct_ptr_struct_member_return_by_val
    {
        Holder_g_struct_ptr h; h.m = g_struct_buf + 0;
        h.m = return_g_struct_ptr();
        check(h.m == g_struct_buf + 2, "struct_ptr_struct_member_return_by_val");
        std::printf("CASE 588: struct_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 589: struct_ptr_array_elem_assign
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        arr[1] = g_struct_buf + 2;
        check(arr[1] == g_struct_buf + 2, "struct_ptr_array_elem_assign");
        std::printf("CASE 589: struct_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 590: struct_ptr_array_elem_compound_assign
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        arr[1] += 1;
        check(arr[1] == (g_struct_buf + 0) + 1, "struct_ptr_array_elem_compound_assign");
        std::printf("CASE 590: struct_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 591: struct_ptr_array_elem_inc_dec
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        ++arr[1];
        --arr[1];
        check(arr[1] == g_struct_buf + 0, "struct_ptr_array_elem_inc_dec");
        std::printf("CASE 591: struct_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 592: struct_ptr_array_elem_read_after_write
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        arr[1] = g_struct_buf + 2;
        S* read_val = arr[1];
        check(read_val == g_struct_buf + 2, "struct_ptr_array_elem_read_after_write");
        std::printf("CASE 592: struct_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 593: struct_ptr_array_elem_pass_by_val
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        take_val_g_struct_ptr(arr[1]);
        check(arr[1] == g_struct_buf + 0, "struct_ptr_array_elem_pass_by_val");
        std::printf("CASE 593: struct_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 594: struct_ptr_array_elem_pass_by_ref
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        take_ref_g_struct_ptr(arr[1]);
        check(arr[1] == g_struct_buf + 0, "struct_ptr_array_elem_pass_by_ref");
        std::printf("CASE 594: struct_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 595: struct_ptr_array_elem_return_by_val
    {
        S* arr[3] = { g_struct_buf + 0, g_struct_buf + 0, g_struct_buf + 0 };
        arr[1] = return_g_struct_ptr();
        check(arr[1] == g_struct_buf + 2, "struct_ptr_array_elem_return_by_val");
        std::printf("CASE 595: struct_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 596: struct_ptr_via_ref_assign
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        ref = g_struct_buf + 2;
        check(ref == g_struct_buf + 2, "struct_ptr_via_ref_assign");
        std::printf("CASE 596: struct_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 597: struct_ptr_via_ref_compound_assign
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        ref += 1;
        check(ref == (g_struct_buf + 0) + 1, "struct_ptr_via_ref_compound_assign");
        std::printf("CASE 597: struct_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 598: struct_ptr_via_ref_inc_dec
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        ++ref;
        --ref;
        check(ref == g_struct_buf + 0, "struct_ptr_via_ref_inc_dec");
        std::printf("CASE 598: struct_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 599: struct_ptr_via_ref_read_after_write
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        ref = g_struct_buf + 2;
        S* read_val = ref;
        check(read_val == g_struct_buf + 2, "struct_ptr_via_ref_read_after_write");
        std::printf("CASE 599: struct_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 600: struct_ptr_via_ref_pass_by_val
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        take_val_g_struct_ptr(ref);
        check(ref == g_struct_buf + 0, "struct_ptr_via_ref_pass_by_val");
        std::printf("CASE 600: struct_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 601: struct_ptr_via_ref_pass_by_ref
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        take_ref_g_struct_ptr(ref);
        check(ref == g_struct_buf + 0, "struct_ptr_via_ref_pass_by_ref");
        std::printf("CASE 601: struct_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 602: struct_ptr_via_ref_return_by_val
    {
        S* base_val = g_struct_buf + 0;
        S*& ref = base_val;
        ref = return_g_struct_ptr();
        check(ref == g_struct_buf + 2, "struct_ptr_via_ref_return_by_val");
        std::printf("CASE 602: struct_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 603: struct_ptr_via_ptr_assign
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        *ptr = g_struct_buf + 2;
        check(*ptr == g_struct_buf + 2, "struct_ptr_via_ptr_assign");
        std::printf("CASE 603: struct_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 604: struct_ptr_via_ptr_compound_assign
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (g_struct_buf + 0) + 1, "struct_ptr_via_ptr_compound_assign");
        std::printf("CASE 604: struct_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 605: struct_ptr_via_ptr_inc_dec
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == g_struct_buf + 0, "struct_ptr_via_ptr_inc_dec");
        std::printf("CASE 605: struct_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 606: struct_ptr_via_ptr_read_after_write
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        *ptr = g_struct_buf + 2;
        S* read_val = *ptr;
        check(read_val == g_struct_buf + 2, "struct_ptr_via_ptr_read_after_write");
        std::printf("CASE 606: struct_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 607: struct_ptr_via_ptr_pass_by_val
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        take_val_g_struct_ptr(*ptr);
        check(*ptr == g_struct_buf + 0, "struct_ptr_via_ptr_pass_by_val");
        std::printf("CASE 607: struct_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 608: struct_ptr_via_ptr_pass_by_ref
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        take_ref_g_struct_ptr(*ptr);
        check(*ptr == g_struct_buf + 0, "struct_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 608: struct_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 609: struct_ptr_via_ptr_return_by_val
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        *ptr = return_g_struct_ptr();
        check(*ptr == g_struct_buf + 2, "struct_ptr_via_ptr_return_by_val");
        std::printf("CASE 609: struct_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 610: struct_ptr_via_pptr_assign
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr = g_struct_buf + 2;
        check(**pptr == g_struct_buf + 2, "struct_ptr_via_pptr_assign");
        std::printf("CASE 610: struct_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 611: struct_ptr_via_pptr_compound_assign
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (g_struct_buf + 0) + 1, "struct_ptr_via_pptr_compound_assign");
        std::printf("CASE 611: struct_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 612: struct_ptr_via_pptr_inc_dec
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == g_struct_buf + 0, "struct_ptr_via_pptr_inc_dec");
        std::printf("CASE 612: struct_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 613: struct_ptr_via_pptr_read_after_write
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr = g_struct_buf + 2;
        S* read_val = **pptr;
        check(read_val == g_struct_buf + 2, "struct_ptr_via_pptr_read_after_write");
        std::printf("CASE 613: struct_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 614: struct_ptr_via_pptr_pass_by_val
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        take_val_g_struct_ptr(**pptr);
        check(**pptr == g_struct_buf + 0, "struct_ptr_via_pptr_pass_by_val");
        std::printf("CASE 614: struct_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 615: struct_ptr_via_pptr_pass_by_ref
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        take_ref_g_struct_ptr(**pptr);
        check(**pptr == g_struct_buf + 0, "struct_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 615: struct_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 616: struct_ptr_via_pptr_return_by_val
    {
        S* base_val = g_struct_buf + 0;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr = return_g_struct_ptr();
        check(**pptr == g_struct_buf + 2, "struct_ptr_via_pptr_return_by_val");
        std::printf("CASE 616: struct_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 617: enum_color_local_assign
    {
        Color lv = Green;
        lv = Blue;
        check(lv == Blue, "enum_color_local_assign");
        std::printf("CASE 617: enum_color_local_assign value=%d\n", (int)(lv));
    }

    // Case 618: enum_color_local_compound_assign
    {
        Color lv = Green;
        lv = Blue;
        check(lv == Blue, "enum_color_local_compound_assign");
        std::printf("CASE 618: enum_color_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 619: enum_color_local_inc_dec
    {
        Color lv = Green;
        lv = Green;
        check(lv == Green, "enum_color_local_inc_dec");
        std::printf("CASE 619: enum_color_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 620: enum_color_local_read_after_write
    {
        Color lv = Green;
        lv = Blue;
        Color read_val = lv;
        check(read_val == Blue, "enum_color_local_read_after_write");
        std::printf("CASE 620: enum_color_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 621: enum_color_local_pass_by_val
    {
        Color lv = Green;
        take_val_g_enum_color(lv);
        check(lv == Green, "enum_color_local_pass_by_val");
        std::printf("CASE 621: enum_color_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 622: enum_color_local_pass_by_ref
    {
        Color lv = Green;
        take_ref_g_enum_color(lv);
        check(lv == Green, "enum_color_local_pass_by_ref");
        std::printf("CASE 622: enum_color_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 623: enum_color_local_return_by_val
    {
        Color lv = Green;
        lv = return_g_enum_color();
        check(lv == Blue, "enum_color_local_return_by_val");
        std::printf("CASE 623: enum_color_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 624: enum_color_global_assign
    {
        g_var_enum_color = Green;
        g_var_enum_color = Blue;
        check(g_var_enum_color == Blue, "enum_color_global_assign");
        std::printf("CASE 624: enum_color_global_assign value=%d\n", (int)(g_var_enum_color));
    }

    // Case 625: enum_color_global_compound_assign
    {
        g_var_enum_color = Green;
        g_var_enum_color = Blue;
        check(g_var_enum_color == Blue, "enum_color_global_compound_assign");
        std::printf("CASE 625: enum_color_global_compound_assign value=%d\n", (int)(g_var_enum_color));
    }

    // Case 626: enum_color_global_inc_dec
    {
        g_var_enum_color = Green;
        g_var_enum_color = Green;
        check(g_var_enum_color == Green, "enum_color_global_inc_dec");
        std::printf("CASE 626: enum_color_global_inc_dec value=%d\n", (int)(g_var_enum_color));
    }

    // Case 627: enum_color_global_read_after_write
    {
        g_var_enum_color = Green;
        g_var_enum_color = Blue;
        Color read_val = g_var_enum_color;
        check(read_val == Blue, "enum_color_global_read_after_write");
        std::printf("CASE 627: enum_color_global_read_after_write value=%d\n", (int)(g_var_enum_color));
    }

    // Case 628: enum_color_global_pass_by_val
    {
        g_var_enum_color = Green;
        take_val_g_enum_color(g_var_enum_color);
        check(g_var_enum_color == Green, "enum_color_global_pass_by_val");
        std::printf("CASE 628: enum_color_global_pass_by_val value=%d\n", (int)(g_var_enum_color));
    }

    // Case 629: enum_color_global_pass_by_ref
    {
        g_var_enum_color = Green;
        take_ref_g_enum_color(g_var_enum_color);
        check(g_var_enum_color == Green, "enum_color_global_pass_by_ref");
        std::printf("CASE 629: enum_color_global_pass_by_ref value=%d\n", (int)(g_var_enum_color));
    }

    // Case 630: enum_color_global_return_by_val
    {
        g_var_enum_color = Green;
        g_var_enum_color = return_g_enum_color();
        check(g_var_enum_color == Blue, "enum_color_global_return_by_val");
        std::printf("CASE 630: enum_color_global_return_by_val value=%d\n", (int)(g_var_enum_color));
    }

    // Case 631: enum_color_static_local_assign
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        s_var_enum_color = Blue;
        check(s_var_enum_color == Blue, "enum_color_static_local_assign");
        std::printf("CASE 631: enum_color_static_local_assign value=%d\n", (int)(s_var_enum_color));
    }

    // Case 632: enum_color_static_local_compound_assign
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        s_var_enum_color = Blue;
        check(s_var_enum_color == Blue, "enum_color_static_local_compound_assign");
        std::printf("CASE 632: enum_color_static_local_compound_assign value=%d\n", (int)(s_var_enum_color));
    }

    // Case 633: enum_color_static_local_inc_dec
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        s_var_enum_color = Green;
        check(s_var_enum_color == Green, "enum_color_static_local_inc_dec");
        std::printf("CASE 633: enum_color_static_local_inc_dec value=%d\n", (int)(s_var_enum_color));
    }

    // Case 634: enum_color_static_local_read_after_write
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        s_var_enum_color = Blue;
        Color read_val = s_var_enum_color;
        check(read_val == Blue, "enum_color_static_local_read_after_write");
        std::printf("CASE 634: enum_color_static_local_read_after_write value=%d\n", (int)(s_var_enum_color));
    }

    // Case 635: enum_color_static_local_pass_by_val
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        take_val_g_enum_color(s_var_enum_color);
        check(s_var_enum_color == Green, "enum_color_static_local_pass_by_val");
        std::printf("CASE 635: enum_color_static_local_pass_by_val value=%d\n", (int)(s_var_enum_color));
    }

    // Case 636: enum_color_static_local_pass_by_ref
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        take_ref_g_enum_color(s_var_enum_color);
        check(s_var_enum_color == Green, "enum_color_static_local_pass_by_ref");
        std::printf("CASE 636: enum_color_static_local_pass_by_ref value=%d\n", (int)(s_var_enum_color));
    }

    // Case 637: enum_color_static_local_return_by_val
    {
        static Color s_var_enum_color; s_var_enum_color = Green;
        s_var_enum_color = return_g_enum_color();
        check(s_var_enum_color == Blue, "enum_color_static_local_return_by_val");
        std::printf("CASE 637: enum_color_static_local_return_by_val value=%d\n", (int)(s_var_enum_color));
    }

    // Case 638: enum_color_struct_member_assign
    {
        Holder_g_enum_color h; h.m = Green;
        h.m = Blue;
        check(h.m == Blue, "enum_color_struct_member_assign");
        std::printf("CASE 638: enum_color_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 639: enum_color_struct_member_compound_assign
    {
        Holder_g_enum_color h; h.m = Green;
        h.m = Blue;
        check(h.m == Blue, "enum_color_struct_member_compound_assign");
        std::printf("CASE 639: enum_color_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 640: enum_color_struct_member_inc_dec
    {
        Holder_g_enum_color h; h.m = Green;
        h.m = Green;
        check(h.m == Green, "enum_color_struct_member_inc_dec");
        std::printf("CASE 640: enum_color_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 641: enum_color_struct_member_read_after_write
    {
        Holder_g_enum_color h; h.m = Green;
        h.m = Blue;
        Color read_val = h.m;
        check(read_val == Blue, "enum_color_struct_member_read_after_write");
        std::printf("CASE 641: enum_color_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 642: enum_color_struct_member_pass_by_val
    {
        Holder_g_enum_color h; h.m = Green;
        take_val_g_enum_color(h.m);
        check(h.m == Green, "enum_color_struct_member_pass_by_val");
        std::printf("CASE 642: enum_color_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 643: enum_color_struct_member_pass_by_ref
    {
        Holder_g_enum_color h; h.m = Green;
        take_ref_g_enum_color(h.m);
        check(h.m == Green, "enum_color_struct_member_pass_by_ref");
        std::printf("CASE 643: enum_color_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 644: enum_color_struct_member_return_by_val
    {
        Holder_g_enum_color h; h.m = Green;
        h.m = return_g_enum_color();
        check(h.m == Blue, "enum_color_struct_member_return_by_val");
        std::printf("CASE 644: enum_color_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 645: enum_color_array_elem_assign
    {
        Color arr[3] = { Green, Green, Green };
        arr[1] = Blue;
        check(arr[1] == Blue, "enum_color_array_elem_assign");
        std::printf("CASE 645: enum_color_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 646: enum_color_array_elem_compound_assign
    {
        Color arr[3] = { Green, Green, Green };
        arr[1] = Blue;
        check(arr[1] == Blue, "enum_color_array_elem_compound_assign");
        std::printf("CASE 646: enum_color_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 647: enum_color_array_elem_inc_dec
    {
        Color arr[3] = { Green, Green, Green };
        arr[1] = Green;
        check(arr[1] == Green, "enum_color_array_elem_inc_dec");
        std::printf("CASE 647: enum_color_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 648: enum_color_array_elem_read_after_write
    {
        Color arr[3] = { Green, Green, Green };
        arr[1] = Blue;
        Color read_val = arr[1];
        check(read_val == Blue, "enum_color_array_elem_read_after_write");
        std::printf("CASE 648: enum_color_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 649: enum_color_array_elem_pass_by_val
    {
        Color arr[3] = { Green, Green, Green };
        take_val_g_enum_color(arr[1]);
        check(arr[1] == Green, "enum_color_array_elem_pass_by_val");
        std::printf("CASE 649: enum_color_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 650: enum_color_array_elem_pass_by_ref
    {
        Color arr[3] = { Green, Green, Green };
        take_ref_g_enum_color(arr[1]);
        check(arr[1] == Green, "enum_color_array_elem_pass_by_ref");
        std::printf("CASE 650: enum_color_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 651: enum_color_array_elem_return_by_val
    {
        Color arr[3] = { Green, Green, Green };
        arr[1] = return_g_enum_color();
        check(arr[1] == Blue, "enum_color_array_elem_return_by_val");
        std::printf("CASE 651: enum_color_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 652: enum_color_via_ref_assign
    {
        Color base_val = Green;
        Color& ref = base_val;
        ref = Blue;
        check(ref == Blue, "enum_color_via_ref_assign");
        std::printf("CASE 652: enum_color_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 653: enum_color_via_ref_compound_assign
    {
        Color base_val = Green;
        Color& ref = base_val;
        ref = Blue;
        check(ref == Blue, "enum_color_via_ref_compound_assign");
        std::printf("CASE 653: enum_color_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 654: enum_color_via_ref_inc_dec
    {
        Color base_val = Green;
        Color& ref = base_val;
        ref = Green;
        check(ref == Green, "enum_color_via_ref_inc_dec");
        std::printf("CASE 654: enum_color_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 655: enum_color_via_ref_read_after_write
    {
        Color base_val = Green;
        Color& ref = base_val;
        ref = Blue;
        Color read_val = ref;
        check(read_val == Blue, "enum_color_via_ref_read_after_write");
        std::printf("CASE 655: enum_color_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 656: enum_color_via_ref_pass_by_val
    {
        Color base_val = Green;
        Color& ref = base_val;
        take_val_g_enum_color(ref);
        check(ref == Green, "enum_color_via_ref_pass_by_val");
        std::printf("CASE 656: enum_color_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 657: enum_color_via_ref_pass_by_ref
    {
        Color base_val = Green;
        Color& ref = base_val;
        take_ref_g_enum_color(ref);
        check(ref == Green, "enum_color_via_ref_pass_by_ref");
        std::printf("CASE 657: enum_color_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 658: enum_color_via_ref_return_by_val
    {
        Color base_val = Green;
        Color& ref = base_val;
        ref = return_g_enum_color();
        check(ref == Blue, "enum_color_via_ref_return_by_val");
        std::printf("CASE 658: enum_color_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 659: enum_color_via_ptr_assign
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        *ptr = Blue;
        check(*ptr == Blue, "enum_color_via_ptr_assign");
        std::printf("CASE 659: enum_color_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 660: enum_color_via_ptr_compound_assign
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        *ptr = Blue;
        check(*ptr == Blue, "enum_color_via_ptr_compound_assign");
        std::printf("CASE 660: enum_color_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 661: enum_color_via_ptr_inc_dec
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        *ptr = Green;
        check(*ptr == Green, "enum_color_via_ptr_inc_dec");
        std::printf("CASE 661: enum_color_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 662: enum_color_via_ptr_read_after_write
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        *ptr = Blue;
        Color read_val = *ptr;
        check(read_val == Blue, "enum_color_via_ptr_read_after_write");
        std::printf("CASE 662: enum_color_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 663: enum_color_via_ptr_pass_by_val
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        take_val_g_enum_color(*ptr);
        check(*ptr == Green, "enum_color_via_ptr_pass_by_val");
        std::printf("CASE 663: enum_color_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 664: enum_color_via_ptr_pass_by_ref
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        take_ref_g_enum_color(*ptr);
        check(*ptr == Green, "enum_color_via_ptr_pass_by_ref");
        std::printf("CASE 664: enum_color_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 665: enum_color_via_ptr_return_by_val
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        *ptr = return_g_enum_color();
        check(*ptr == Blue, "enum_color_via_ptr_return_by_val");
        std::printf("CASE 665: enum_color_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 666: enum_color_via_pptr_assign
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Blue;
        check(**pptr == Blue, "enum_color_via_pptr_assign");
        std::printf("CASE 666: enum_color_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 667: enum_color_via_pptr_compound_assign
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Blue;
        check(**pptr == Blue, "enum_color_via_pptr_compound_assign");
        std::printf("CASE 667: enum_color_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 668: enum_color_via_pptr_inc_dec
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Green;
        check(**pptr == Green, "enum_color_via_pptr_inc_dec");
        std::printf("CASE 668: enum_color_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 669: enum_color_via_pptr_read_after_write
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Blue;
        Color read_val = **pptr;
        check(read_val == Blue, "enum_color_via_pptr_read_after_write");
        std::printf("CASE 669: enum_color_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 670: enum_color_via_pptr_pass_by_val
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        take_val_g_enum_color(**pptr);
        check(**pptr == Green, "enum_color_via_pptr_pass_by_val");
        std::printf("CASE 670: enum_color_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 671: enum_color_via_pptr_pass_by_ref
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        take_ref_g_enum_color(**pptr);
        check(**pptr == Green, "enum_color_via_pptr_pass_by_ref");
        std::printf("CASE 671: enum_color_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 672: enum_color_via_pptr_return_by_val
    {
        Color base_val = Green;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = return_g_enum_color();
        check(**pptr == Blue, "enum_color_via_pptr_return_by_val");
        std::printf("CASE 672: enum_color_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    std::printf("Total test cases executed: %d, verified checks: %d, rolling hash: %u\n", 672, total_checks, rolling_hash);
    return 0;
}