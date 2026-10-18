#include <cstdio>

enum Color { Red = 0, Green = 1, Blue = 2 };
struct S { int id; };

char h_char_buf[16] = { 'z', 'y', 'x', 'w', 'v', 'u', 't', 's', 'r', 'q', 'p', 'o', 'n', 'm', 'l', 0 };
int h_int_buf[8] = {100, 200, 300, 400, 500, 600, 700, 800};
double h_double_buf[8] = {1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5};
S h_struct_buf[8] = {{10}, {20}, {30}, {40}, {50}, {60}, {70}, {80}};

char h_var_char = 'C';
struct Holder_h_char { char m; };
Holder_h_char h_holder_char = { 'C' };
char h_arr_char[4] = { 'C', 'C', 'D', 'D' };
void take_val_h_char(char x) { (void)x; }
void take_ref_h_char(char& x) { (void)x; }
char return_h_char() { return 'D'; }

short h_var_short = 20;
struct Holder_h_short { short m; };
Holder_h_short h_holder_short = { 20 };
short h_arr_short[4] = { 20, 20, 40, 40 };
void take_val_h_short(short x) { (void)x; }
void take_ref_h_short(short& x) { (void)x; }
short return_h_short() { return 40; }

int h_var_int = 301;
struct Holder_h_int { int m; };
Holder_h_int h_holder_int = { 301 };
int h_arr_int[4] = { 301, 301, 402, 402 };
void take_val_h_int(int x) { (void)x; }
void take_ref_h_int(int& x) { (void)x; }
int return_h_int() { return 402; }

long long h_var_long_long = 3000000000LL;
struct Holder_h_long_long { long long m; };
Holder_h_long_long h_holder_long_long = { 3000000000LL };
long long h_arr_long_long[4] = { 3000000000LL, 3000000000LL, 4000000000LL, 4000000000LL };
void take_val_h_long_long(long long x) { (void)x; }
void take_ref_h_long_long(long long& x) { (void)x; }
long long return_h_long_long() { return 4000000000LL; }

float h_var_float = 2.5f;
struct Holder_h_float { float m; };
Holder_h_float h_holder_float = { 2.5f };
float h_arr_float[4] = { 2.5f, 2.5f, 6.5f, 6.5f };
void take_val_h_float(float x) { (void)x; }
void take_ref_h_float(float& x) { (void)x; }
float return_h_float() { return 6.5f; }

double h_var_double = 3.25;
struct Holder_h_double { double m; };
Holder_h_double h_holder_double = { 3.25 };
double h_arr_double[4] = { 3.25, 3.25, 7.75, 7.75 };
void take_val_h_double(double x) { (void)x; }
void take_ref_h_double(double& x) { (void)x; }
double return_h_double() { return 7.75; }

bool h_var_bool = false;
struct Holder_h_bool { bool m; };
Holder_h_bool h_holder_bool = { false };
bool h_arr_bool[4] = { false, false, true, true };
void take_val_h_bool(bool x) { (void)x; }
void take_ref_h_bool(bool& x) { (void)x; }
bool return_h_bool() { return true; }

char* h_var_char_ptr = h_char_buf + 1;
struct Holder_h_char_ptr { char* m; };
Holder_h_char_ptr h_holder_char_ptr = { h_char_buf + 1 };
char* h_arr_char_ptr[4] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 3, h_char_buf + 3 };
void take_val_h_char_ptr(char* x) { (void)x; }
void take_ref_h_char_ptr(char*& x) { (void)x; }
char* return_h_char_ptr() { return h_char_buf + 3; }

int* h_var_int_ptr = h_int_buf + 1;
struct Holder_h_int_ptr { int* m; };
Holder_h_int_ptr h_holder_int_ptr = { h_int_buf + 1 };
int* h_arr_int_ptr[4] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 3, h_int_buf + 3 };
void take_val_h_int_ptr(int* x) { (void)x; }
void take_ref_h_int_ptr(int*& x) { (void)x; }
int* return_h_int_ptr() { return h_int_buf + 3; }

double* h_var_double_ptr = h_double_buf + 1;
struct Holder_h_double_ptr { double* m; };
Holder_h_double_ptr h_holder_double_ptr = { h_double_buf + 1 };
double* h_arr_double_ptr[4] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 3, h_double_buf + 3 };
void take_val_h_double_ptr(double* x) { (void)x; }
void take_ref_h_double_ptr(double*& x) { (void)x; }
double* return_h_double_ptr() { return h_double_buf + 3; }

S* h_var_struct_ptr = h_struct_buf + 1;
struct Holder_h_struct_ptr { S* m; };
Holder_h_struct_ptr h_holder_struct_ptr = { h_struct_buf + 1 };
S* h_arr_struct_ptr[4] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 3, h_struct_buf + 3 };
void take_val_h_struct_ptr(S* x) { (void)x; }
void take_ref_h_struct_ptr(S*& x) { (void)x; }
S* return_h_struct_ptr() { return h_struct_buf + 3; }

Color h_var_enum_color = Red;
struct Holder_h_enum_color { Color m; };
Holder_h_enum_color h_holder_enum_color = { Red };
Color h_arr_enum_color[4] = { Red, Red, Blue, Blue };
void take_val_h_enum_color(Color x) { (void)x; }
void take_ref_h_enum_color(Color& x) { (void)x; }
Color return_h_enum_color() { return Blue; }

int total_checks = 0;
unsigned int rolling_hash = 2654435761u;
void check(bool condition, const char* msg) {
    total_checks++;
    rolling_hash = rolling_hash * 16777619u ^ (condition ? 1u : 0u);
    if (!condition) {
        std::printf("FAILED: %s\n", msg);
    }
}

int main() {
    // Case 1: heldout_char_local_assign
    {
        char lv = 'C';
        lv = 'D';
        check(lv == 'D', "heldout_char_local_assign");
        std::printf("CASE 1: heldout_char_local_assign value=%d\n", (int)(lv));
    }

    // Case 2: heldout_char_local_compound_assign
    {
        char lv = 'C';
        lv += 1;
        check(lv == ('C') + 1, "heldout_char_local_compound_assign");
        std::printf("CASE 2: heldout_char_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 3: heldout_char_local_inc_dec
    {
        char lv = 'C';
        ++lv;
        --lv;
        check(lv == 'C', "heldout_char_local_inc_dec");
        std::printf("CASE 3: heldout_char_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 4: heldout_char_local_read_after_write
    {
        char lv = 'C';
        lv = 'D';
        char read_val = lv;
        check(read_val == 'D', "heldout_char_local_read_after_write");
        std::printf("CASE 4: heldout_char_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 5: heldout_char_local_pass_by_val
    {
        char lv = 'C';
        take_val_h_char(lv);
        check(lv == 'C', "heldout_char_local_pass_by_val");
        std::printf("CASE 5: heldout_char_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 6: heldout_char_local_pass_by_ref
    {
        char lv = 'C';
        take_ref_h_char(lv);
        check(lv == 'C', "heldout_char_local_pass_by_ref");
        std::printf("CASE 6: heldout_char_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 7: heldout_char_local_return_by_val
    {
        char lv = 'C';
        lv = return_h_char();
        check(lv == 'D', "heldout_char_local_return_by_val");
        std::printf("CASE 7: heldout_char_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 8: heldout_char_global_assign
    {
        h_var_char = 'C';
        h_var_char = 'D';
        check(h_var_char == 'D', "heldout_char_global_assign");
        std::printf("CASE 8: heldout_char_global_assign value=%d\n", (int)(h_var_char));
    }

    // Case 9: heldout_char_global_compound_assign
    {
        h_var_char = 'C';
        h_var_char += 1;
        check(h_var_char == ('C') + 1, "heldout_char_global_compound_assign");
        std::printf("CASE 9: heldout_char_global_compound_assign value=%d\n", (int)(h_var_char));
    }

    // Case 10: heldout_char_global_inc_dec
    {
        h_var_char = 'C';
        ++h_var_char;
        --h_var_char;
        check(h_var_char == 'C', "heldout_char_global_inc_dec");
        std::printf("CASE 10: heldout_char_global_inc_dec value=%d\n", (int)(h_var_char));
    }

    // Case 11: heldout_char_global_read_after_write
    {
        h_var_char = 'C';
        h_var_char = 'D';
        char read_val = h_var_char;
        check(read_val == 'D', "heldout_char_global_read_after_write");
        std::printf("CASE 11: heldout_char_global_read_after_write value=%d\n", (int)(h_var_char));
    }

    // Case 12: heldout_char_global_pass_by_val
    {
        h_var_char = 'C';
        take_val_h_char(h_var_char);
        check(h_var_char == 'C', "heldout_char_global_pass_by_val");
        std::printf("CASE 12: heldout_char_global_pass_by_val value=%d\n", (int)(h_var_char));
    }

    // Case 13: heldout_char_global_pass_by_ref
    {
        h_var_char = 'C';
        take_ref_h_char(h_var_char);
        check(h_var_char == 'C', "heldout_char_global_pass_by_ref");
        std::printf("CASE 13: heldout_char_global_pass_by_ref value=%d\n", (int)(h_var_char));
    }

    // Case 14: heldout_char_global_return_by_val
    {
        h_var_char = 'C';
        h_var_char = return_h_char();
        check(h_var_char == 'D', "heldout_char_global_return_by_val");
        std::printf("CASE 14: heldout_char_global_return_by_val value=%d\n", (int)(h_var_char));
    }

    // Case 15: heldout_char_static_local_assign
    {
        static char s_var_char; s_var_char = 'C';
        s_var_char = 'D';
        check(s_var_char == 'D', "heldout_char_static_local_assign");
        std::printf("CASE 15: heldout_char_static_local_assign value=%d\n", (int)(s_var_char));
    }

    // Case 16: heldout_char_static_local_compound_assign
    {
        static char s_var_char; s_var_char = 'C';
        s_var_char += 1;
        check(s_var_char == ('C') + 1, "heldout_char_static_local_compound_assign");
        std::printf("CASE 16: heldout_char_static_local_compound_assign value=%d\n", (int)(s_var_char));
    }

    // Case 17: heldout_char_static_local_inc_dec
    {
        static char s_var_char; s_var_char = 'C';
        ++s_var_char;
        --s_var_char;
        check(s_var_char == 'C', "heldout_char_static_local_inc_dec");
        std::printf("CASE 17: heldout_char_static_local_inc_dec value=%d\n", (int)(s_var_char));
    }

    // Case 18: heldout_char_static_local_read_after_write
    {
        static char s_var_char; s_var_char = 'C';
        s_var_char = 'D';
        char read_val = s_var_char;
        check(read_val == 'D', "heldout_char_static_local_read_after_write");
        std::printf("CASE 18: heldout_char_static_local_read_after_write value=%d\n", (int)(s_var_char));
    }

    // Case 19: heldout_char_static_local_pass_by_val
    {
        static char s_var_char; s_var_char = 'C';
        take_val_h_char(s_var_char);
        check(s_var_char == 'C', "heldout_char_static_local_pass_by_val");
        std::printf("CASE 19: heldout_char_static_local_pass_by_val value=%d\n", (int)(s_var_char));
    }

    // Case 20: heldout_char_static_local_pass_by_ref
    {
        static char s_var_char; s_var_char = 'C';
        take_ref_h_char(s_var_char);
        check(s_var_char == 'C', "heldout_char_static_local_pass_by_ref");
        std::printf("CASE 20: heldout_char_static_local_pass_by_ref value=%d\n", (int)(s_var_char));
    }

    // Case 21: heldout_char_static_local_return_by_val
    {
        static char s_var_char; s_var_char = 'C';
        s_var_char = return_h_char();
        check(s_var_char == 'D', "heldout_char_static_local_return_by_val");
        std::printf("CASE 21: heldout_char_static_local_return_by_val value=%d\n", (int)(s_var_char));
    }

    // Case 22: heldout_char_struct_member_assign
    {
        Holder_h_char h; h.m = 'C';
        h.m = 'D';
        check(h.m == 'D', "heldout_char_struct_member_assign");
        std::printf("CASE 22: heldout_char_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 23: heldout_char_struct_member_compound_assign
    {
        Holder_h_char h; h.m = 'C';
        h.m += 1;
        check(h.m == ('C') + 1, "heldout_char_struct_member_compound_assign");
        std::printf("CASE 23: heldout_char_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 24: heldout_char_struct_member_inc_dec
    {
        Holder_h_char h; h.m = 'C';
        ++h.m;
        --h.m;
        check(h.m == 'C', "heldout_char_struct_member_inc_dec");
        std::printf("CASE 24: heldout_char_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 25: heldout_char_struct_member_read_after_write
    {
        Holder_h_char h; h.m = 'C';
        h.m = 'D';
        char read_val = h.m;
        check(read_val == 'D', "heldout_char_struct_member_read_after_write");
        std::printf("CASE 25: heldout_char_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 26: heldout_char_struct_member_pass_by_val
    {
        Holder_h_char h; h.m = 'C';
        take_val_h_char(h.m);
        check(h.m == 'C', "heldout_char_struct_member_pass_by_val");
        std::printf("CASE 26: heldout_char_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 27: heldout_char_struct_member_pass_by_ref
    {
        Holder_h_char h; h.m = 'C';
        take_ref_h_char(h.m);
        check(h.m == 'C', "heldout_char_struct_member_pass_by_ref");
        std::printf("CASE 27: heldout_char_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 28: heldout_char_struct_member_return_by_val
    {
        Holder_h_char h; h.m = 'C';
        h.m = return_h_char();
        check(h.m == 'D', "heldout_char_struct_member_return_by_val");
        std::printf("CASE 28: heldout_char_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 29: heldout_char_array_elem_assign
    {
        char arr[3] = { 'C', 'C', 'C' };
        arr[1] = 'D';
        check(arr[1] == 'D', "heldout_char_array_elem_assign");
        std::printf("CASE 29: heldout_char_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 30: heldout_char_array_elem_compound_assign
    {
        char arr[3] = { 'C', 'C', 'C' };
        arr[1] += 1;
        check(arr[1] == ('C') + 1, "heldout_char_array_elem_compound_assign");
        std::printf("CASE 30: heldout_char_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 31: heldout_char_array_elem_inc_dec
    {
        char arr[3] = { 'C', 'C', 'C' };
        ++arr[1];
        --arr[1];
        check(arr[1] == 'C', "heldout_char_array_elem_inc_dec");
        std::printf("CASE 31: heldout_char_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 32: heldout_char_array_elem_read_after_write
    {
        char arr[3] = { 'C', 'C', 'C' };
        arr[1] = 'D';
        char read_val = arr[1];
        check(read_val == 'D', "heldout_char_array_elem_read_after_write");
        std::printf("CASE 32: heldout_char_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 33: heldout_char_array_elem_pass_by_val
    {
        char arr[3] = { 'C', 'C', 'C' };
        take_val_h_char(arr[1]);
        check(arr[1] == 'C', "heldout_char_array_elem_pass_by_val");
        std::printf("CASE 33: heldout_char_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 34: heldout_char_array_elem_pass_by_ref
    {
        char arr[3] = { 'C', 'C', 'C' };
        take_ref_h_char(arr[1]);
        check(arr[1] == 'C', "heldout_char_array_elem_pass_by_ref");
        std::printf("CASE 34: heldout_char_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 35: heldout_char_array_elem_return_by_val
    {
        char arr[3] = { 'C', 'C', 'C' };
        arr[1] = return_h_char();
        check(arr[1] == 'D', "heldout_char_array_elem_return_by_val");
        std::printf("CASE 35: heldout_char_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 36: heldout_char_via_ref_assign
    {
        char base_val = 'C';
        char& ref = base_val;
        ref = 'D';
        check(ref == 'D', "heldout_char_via_ref_assign");
        std::printf("CASE 36: heldout_char_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 37: heldout_char_via_ref_compound_assign
    {
        char base_val = 'C';
        char& ref = base_val;
        ref += 1;
        check(ref == ('C') + 1, "heldout_char_via_ref_compound_assign");
        std::printf("CASE 37: heldout_char_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 38: heldout_char_via_ref_inc_dec
    {
        char base_val = 'C';
        char& ref = base_val;
        ++ref;
        --ref;
        check(ref == 'C', "heldout_char_via_ref_inc_dec");
        std::printf("CASE 38: heldout_char_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 39: heldout_char_via_ref_read_after_write
    {
        char base_val = 'C';
        char& ref = base_val;
        ref = 'D';
        char read_val = ref;
        check(read_val == 'D', "heldout_char_via_ref_read_after_write");
        std::printf("CASE 39: heldout_char_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 40: heldout_char_via_ref_pass_by_val
    {
        char base_val = 'C';
        char& ref = base_val;
        take_val_h_char(ref);
        check(ref == 'C', "heldout_char_via_ref_pass_by_val");
        std::printf("CASE 40: heldout_char_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 41: heldout_char_via_ref_pass_by_ref
    {
        char base_val = 'C';
        char& ref = base_val;
        take_ref_h_char(ref);
        check(ref == 'C', "heldout_char_via_ref_pass_by_ref");
        std::printf("CASE 41: heldout_char_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 42: heldout_char_via_ref_return_by_val
    {
        char base_val = 'C';
        char& ref = base_val;
        ref = return_h_char();
        check(ref == 'D', "heldout_char_via_ref_return_by_val");
        std::printf("CASE 42: heldout_char_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 43: heldout_char_via_ptr_assign
    {
        char base_val = 'C';
        char* ptr = &base_val;
        *ptr = 'D';
        check(*ptr == 'D', "heldout_char_via_ptr_assign");
        std::printf("CASE 43: heldout_char_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 44: heldout_char_via_ptr_compound_assign
    {
        char base_val = 'C';
        char* ptr = &base_val;
        *ptr += 1;
        check(*ptr == ('C') + 1, "heldout_char_via_ptr_compound_assign");
        std::printf("CASE 44: heldout_char_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 45: heldout_char_via_ptr_inc_dec
    {
        char base_val = 'C';
        char* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 'C', "heldout_char_via_ptr_inc_dec");
        std::printf("CASE 45: heldout_char_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 46: heldout_char_via_ptr_read_after_write
    {
        char base_val = 'C';
        char* ptr = &base_val;
        *ptr = 'D';
        char read_val = *ptr;
        check(read_val == 'D', "heldout_char_via_ptr_read_after_write");
        std::printf("CASE 46: heldout_char_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 47: heldout_char_via_ptr_pass_by_val
    {
        char base_val = 'C';
        char* ptr = &base_val;
        take_val_h_char(*ptr);
        check(*ptr == 'C', "heldout_char_via_ptr_pass_by_val");
        std::printf("CASE 47: heldout_char_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 48: heldout_char_via_ptr_pass_by_ref
    {
        char base_val = 'C';
        char* ptr = &base_val;
        take_ref_h_char(*ptr);
        check(*ptr == 'C', "heldout_char_via_ptr_pass_by_ref");
        std::printf("CASE 48: heldout_char_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 49: heldout_char_via_ptr_return_by_val
    {
        char base_val = 'C';
        char* ptr = &base_val;
        *ptr = return_h_char();
        check(*ptr == 'D', "heldout_char_via_ptr_return_by_val");
        std::printf("CASE 49: heldout_char_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 50: heldout_char_via_pptr_assign
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr = 'D';
        check(**pptr == 'D', "heldout_char_via_pptr_assign");
        std::printf("CASE 50: heldout_char_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 51: heldout_char_via_pptr_compound_assign
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr += 1;
        check(**pptr == ('C') + 1, "heldout_char_via_pptr_compound_assign");
        std::printf("CASE 51: heldout_char_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 52: heldout_char_via_pptr_inc_dec
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 'C', "heldout_char_via_pptr_inc_dec");
        std::printf("CASE 52: heldout_char_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 53: heldout_char_via_pptr_read_after_write
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr = 'D';
        char read_val = **pptr;
        check(read_val == 'D', "heldout_char_via_pptr_read_after_write");
        std::printf("CASE 53: heldout_char_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 54: heldout_char_via_pptr_pass_by_val
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        take_val_h_char(**pptr);
        check(**pptr == 'C', "heldout_char_via_pptr_pass_by_val");
        std::printf("CASE 54: heldout_char_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 55: heldout_char_via_pptr_pass_by_ref
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        take_ref_h_char(**pptr);
        check(**pptr == 'C', "heldout_char_via_pptr_pass_by_ref");
        std::printf("CASE 55: heldout_char_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 56: heldout_char_via_pptr_return_by_val
    {
        char base_val = 'C';
        char* ptr = &base_val;
        char** pptr = &ptr;
        **pptr = return_h_char();
        check(**pptr == 'D', "heldout_char_via_pptr_return_by_val");
        std::printf("CASE 56: heldout_char_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 57: heldout_short_local_assign
    {
        short lv = 20;
        lv = 40;
        check(lv == 40, "heldout_short_local_assign");
        std::printf("CASE 57: heldout_short_local_assign value=%d\n", (int)(lv));
    }

    // Case 58: heldout_short_local_compound_assign
    {
        short lv = 20;
        lv += 6;
        check(lv == (20) + 6, "heldout_short_local_compound_assign");
        std::printf("CASE 58: heldout_short_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 59: heldout_short_local_inc_dec
    {
        short lv = 20;
        ++lv;
        --lv;
        check(lv == 20, "heldout_short_local_inc_dec");
        std::printf("CASE 59: heldout_short_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 60: heldout_short_local_read_after_write
    {
        short lv = 20;
        lv = 40;
        short read_val = lv;
        check(read_val == 40, "heldout_short_local_read_after_write");
        std::printf("CASE 60: heldout_short_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 61: heldout_short_local_pass_by_val
    {
        short lv = 20;
        take_val_h_short(lv);
        check(lv == 20, "heldout_short_local_pass_by_val");
        std::printf("CASE 61: heldout_short_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 62: heldout_short_local_pass_by_ref
    {
        short lv = 20;
        take_ref_h_short(lv);
        check(lv == 20, "heldout_short_local_pass_by_ref");
        std::printf("CASE 62: heldout_short_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 63: heldout_short_local_return_by_val
    {
        short lv = 20;
        lv = return_h_short();
        check(lv == 40, "heldout_short_local_return_by_val");
        std::printf("CASE 63: heldout_short_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 64: heldout_short_global_assign
    {
        h_var_short = 20;
        h_var_short = 40;
        check(h_var_short == 40, "heldout_short_global_assign");
        std::printf("CASE 64: heldout_short_global_assign value=%d\n", (int)(h_var_short));
    }

    // Case 65: heldout_short_global_compound_assign
    {
        h_var_short = 20;
        h_var_short += 6;
        check(h_var_short == (20) + 6, "heldout_short_global_compound_assign");
        std::printf("CASE 65: heldout_short_global_compound_assign value=%d\n", (int)(h_var_short));
    }

    // Case 66: heldout_short_global_inc_dec
    {
        h_var_short = 20;
        ++h_var_short;
        --h_var_short;
        check(h_var_short == 20, "heldout_short_global_inc_dec");
        std::printf("CASE 66: heldout_short_global_inc_dec value=%d\n", (int)(h_var_short));
    }

    // Case 67: heldout_short_global_read_after_write
    {
        h_var_short = 20;
        h_var_short = 40;
        short read_val = h_var_short;
        check(read_val == 40, "heldout_short_global_read_after_write");
        std::printf("CASE 67: heldout_short_global_read_after_write value=%d\n", (int)(h_var_short));
    }

    // Case 68: heldout_short_global_pass_by_val
    {
        h_var_short = 20;
        take_val_h_short(h_var_short);
        check(h_var_short == 20, "heldout_short_global_pass_by_val");
        std::printf("CASE 68: heldout_short_global_pass_by_val value=%d\n", (int)(h_var_short));
    }

    // Case 69: heldout_short_global_pass_by_ref
    {
        h_var_short = 20;
        take_ref_h_short(h_var_short);
        check(h_var_short == 20, "heldout_short_global_pass_by_ref");
        std::printf("CASE 69: heldout_short_global_pass_by_ref value=%d\n", (int)(h_var_short));
    }

    // Case 70: heldout_short_global_return_by_val
    {
        h_var_short = 20;
        h_var_short = return_h_short();
        check(h_var_short == 40, "heldout_short_global_return_by_val");
        std::printf("CASE 70: heldout_short_global_return_by_val value=%d\n", (int)(h_var_short));
    }

    // Case 71: heldout_short_static_local_assign
    {
        static short s_var_short; s_var_short = 20;
        s_var_short = 40;
        check(s_var_short == 40, "heldout_short_static_local_assign");
        std::printf("CASE 71: heldout_short_static_local_assign value=%d\n", (int)(s_var_short));
    }

    // Case 72: heldout_short_static_local_compound_assign
    {
        static short s_var_short; s_var_short = 20;
        s_var_short += 6;
        check(s_var_short == (20) + 6, "heldout_short_static_local_compound_assign");
        std::printf("CASE 72: heldout_short_static_local_compound_assign value=%d\n", (int)(s_var_short));
    }

    // Case 73: heldout_short_static_local_inc_dec
    {
        static short s_var_short; s_var_short = 20;
        ++s_var_short;
        --s_var_short;
        check(s_var_short == 20, "heldout_short_static_local_inc_dec");
        std::printf("CASE 73: heldout_short_static_local_inc_dec value=%d\n", (int)(s_var_short));
    }

    // Case 74: heldout_short_static_local_read_after_write
    {
        static short s_var_short; s_var_short = 20;
        s_var_short = 40;
        short read_val = s_var_short;
        check(read_val == 40, "heldout_short_static_local_read_after_write");
        std::printf("CASE 74: heldout_short_static_local_read_after_write value=%d\n", (int)(s_var_short));
    }

    // Case 75: heldout_short_static_local_pass_by_val
    {
        static short s_var_short; s_var_short = 20;
        take_val_h_short(s_var_short);
        check(s_var_short == 20, "heldout_short_static_local_pass_by_val");
        std::printf("CASE 75: heldout_short_static_local_pass_by_val value=%d\n", (int)(s_var_short));
    }

    // Case 76: heldout_short_static_local_pass_by_ref
    {
        static short s_var_short; s_var_short = 20;
        take_ref_h_short(s_var_short);
        check(s_var_short == 20, "heldout_short_static_local_pass_by_ref");
        std::printf("CASE 76: heldout_short_static_local_pass_by_ref value=%d\n", (int)(s_var_short));
    }

    // Case 77: heldout_short_static_local_return_by_val
    {
        static short s_var_short; s_var_short = 20;
        s_var_short = return_h_short();
        check(s_var_short == 40, "heldout_short_static_local_return_by_val");
        std::printf("CASE 77: heldout_short_static_local_return_by_val value=%d\n", (int)(s_var_short));
    }

    // Case 78: heldout_short_struct_member_assign
    {
        Holder_h_short h; h.m = 20;
        h.m = 40;
        check(h.m == 40, "heldout_short_struct_member_assign");
        std::printf("CASE 78: heldout_short_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 79: heldout_short_struct_member_compound_assign
    {
        Holder_h_short h; h.m = 20;
        h.m += 6;
        check(h.m == (20) + 6, "heldout_short_struct_member_compound_assign");
        std::printf("CASE 79: heldout_short_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 80: heldout_short_struct_member_inc_dec
    {
        Holder_h_short h; h.m = 20;
        ++h.m;
        --h.m;
        check(h.m == 20, "heldout_short_struct_member_inc_dec");
        std::printf("CASE 80: heldout_short_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 81: heldout_short_struct_member_read_after_write
    {
        Holder_h_short h; h.m = 20;
        h.m = 40;
        short read_val = h.m;
        check(read_val == 40, "heldout_short_struct_member_read_after_write");
        std::printf("CASE 81: heldout_short_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 82: heldout_short_struct_member_pass_by_val
    {
        Holder_h_short h; h.m = 20;
        take_val_h_short(h.m);
        check(h.m == 20, "heldout_short_struct_member_pass_by_val");
        std::printf("CASE 82: heldout_short_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 83: heldout_short_struct_member_pass_by_ref
    {
        Holder_h_short h; h.m = 20;
        take_ref_h_short(h.m);
        check(h.m == 20, "heldout_short_struct_member_pass_by_ref");
        std::printf("CASE 83: heldout_short_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 84: heldout_short_struct_member_return_by_val
    {
        Holder_h_short h; h.m = 20;
        h.m = return_h_short();
        check(h.m == 40, "heldout_short_struct_member_return_by_val");
        std::printf("CASE 84: heldout_short_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 85: heldout_short_array_elem_assign
    {
        short arr[3] = { 20, 20, 20 };
        arr[1] = 40;
        check(arr[1] == 40, "heldout_short_array_elem_assign");
        std::printf("CASE 85: heldout_short_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 86: heldout_short_array_elem_compound_assign
    {
        short arr[3] = { 20, 20, 20 };
        arr[1] += 6;
        check(arr[1] == (20) + 6, "heldout_short_array_elem_compound_assign");
        std::printf("CASE 86: heldout_short_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 87: heldout_short_array_elem_inc_dec
    {
        short arr[3] = { 20, 20, 20 };
        ++arr[1];
        --arr[1];
        check(arr[1] == 20, "heldout_short_array_elem_inc_dec");
        std::printf("CASE 87: heldout_short_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 88: heldout_short_array_elem_read_after_write
    {
        short arr[3] = { 20, 20, 20 };
        arr[1] = 40;
        short read_val = arr[1];
        check(read_val == 40, "heldout_short_array_elem_read_after_write");
        std::printf("CASE 88: heldout_short_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 89: heldout_short_array_elem_pass_by_val
    {
        short arr[3] = { 20, 20, 20 };
        take_val_h_short(arr[1]);
        check(arr[1] == 20, "heldout_short_array_elem_pass_by_val");
        std::printf("CASE 89: heldout_short_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 90: heldout_short_array_elem_pass_by_ref
    {
        short arr[3] = { 20, 20, 20 };
        take_ref_h_short(arr[1]);
        check(arr[1] == 20, "heldout_short_array_elem_pass_by_ref");
        std::printf("CASE 90: heldout_short_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 91: heldout_short_array_elem_return_by_val
    {
        short arr[3] = { 20, 20, 20 };
        arr[1] = return_h_short();
        check(arr[1] == 40, "heldout_short_array_elem_return_by_val");
        std::printf("CASE 91: heldout_short_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 92: heldout_short_via_ref_assign
    {
        short base_val = 20;
        short& ref = base_val;
        ref = 40;
        check(ref == 40, "heldout_short_via_ref_assign");
        std::printf("CASE 92: heldout_short_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 93: heldout_short_via_ref_compound_assign
    {
        short base_val = 20;
        short& ref = base_val;
        ref += 6;
        check(ref == (20) + 6, "heldout_short_via_ref_compound_assign");
        std::printf("CASE 93: heldout_short_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 94: heldout_short_via_ref_inc_dec
    {
        short base_val = 20;
        short& ref = base_val;
        ++ref;
        --ref;
        check(ref == 20, "heldout_short_via_ref_inc_dec");
        std::printf("CASE 94: heldout_short_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 95: heldout_short_via_ref_read_after_write
    {
        short base_val = 20;
        short& ref = base_val;
        ref = 40;
        short read_val = ref;
        check(read_val == 40, "heldout_short_via_ref_read_after_write");
        std::printf("CASE 95: heldout_short_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 96: heldout_short_via_ref_pass_by_val
    {
        short base_val = 20;
        short& ref = base_val;
        take_val_h_short(ref);
        check(ref == 20, "heldout_short_via_ref_pass_by_val");
        std::printf("CASE 96: heldout_short_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 97: heldout_short_via_ref_pass_by_ref
    {
        short base_val = 20;
        short& ref = base_val;
        take_ref_h_short(ref);
        check(ref == 20, "heldout_short_via_ref_pass_by_ref");
        std::printf("CASE 97: heldout_short_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 98: heldout_short_via_ref_return_by_val
    {
        short base_val = 20;
        short& ref = base_val;
        ref = return_h_short();
        check(ref == 40, "heldout_short_via_ref_return_by_val");
        std::printf("CASE 98: heldout_short_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 99: heldout_short_via_ptr_assign
    {
        short base_val = 20;
        short* ptr = &base_val;
        *ptr = 40;
        check(*ptr == 40, "heldout_short_via_ptr_assign");
        std::printf("CASE 99: heldout_short_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 100: heldout_short_via_ptr_compound_assign
    {
        short base_val = 20;
        short* ptr = &base_val;
        *ptr += 6;
        check(*ptr == (20) + 6, "heldout_short_via_ptr_compound_assign");
        std::printf("CASE 100: heldout_short_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 101: heldout_short_via_ptr_inc_dec
    {
        short base_val = 20;
        short* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 20, "heldout_short_via_ptr_inc_dec");
        std::printf("CASE 101: heldout_short_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 102: heldout_short_via_ptr_read_after_write
    {
        short base_val = 20;
        short* ptr = &base_val;
        *ptr = 40;
        short read_val = *ptr;
        check(read_val == 40, "heldout_short_via_ptr_read_after_write");
        std::printf("CASE 102: heldout_short_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 103: heldout_short_via_ptr_pass_by_val
    {
        short base_val = 20;
        short* ptr = &base_val;
        take_val_h_short(*ptr);
        check(*ptr == 20, "heldout_short_via_ptr_pass_by_val");
        std::printf("CASE 103: heldout_short_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 104: heldout_short_via_ptr_pass_by_ref
    {
        short base_val = 20;
        short* ptr = &base_val;
        take_ref_h_short(*ptr);
        check(*ptr == 20, "heldout_short_via_ptr_pass_by_ref");
        std::printf("CASE 104: heldout_short_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 105: heldout_short_via_ptr_return_by_val
    {
        short base_val = 20;
        short* ptr = &base_val;
        *ptr = return_h_short();
        check(*ptr == 40, "heldout_short_via_ptr_return_by_val");
        std::printf("CASE 105: heldout_short_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 106: heldout_short_via_pptr_assign
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr = 40;
        check(**pptr == 40, "heldout_short_via_pptr_assign");
        std::printf("CASE 106: heldout_short_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 107: heldout_short_via_pptr_compound_assign
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr += 6;
        check(**pptr == (20) + 6, "heldout_short_via_pptr_compound_assign");
        std::printf("CASE 107: heldout_short_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 108: heldout_short_via_pptr_inc_dec
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 20, "heldout_short_via_pptr_inc_dec");
        std::printf("CASE 108: heldout_short_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 109: heldout_short_via_pptr_read_after_write
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr = 40;
        short read_val = **pptr;
        check(read_val == 40, "heldout_short_via_pptr_read_after_write");
        std::printf("CASE 109: heldout_short_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 110: heldout_short_via_pptr_pass_by_val
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        take_val_h_short(**pptr);
        check(**pptr == 20, "heldout_short_via_pptr_pass_by_val");
        std::printf("CASE 110: heldout_short_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 111: heldout_short_via_pptr_pass_by_ref
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        take_ref_h_short(**pptr);
        check(**pptr == 20, "heldout_short_via_pptr_pass_by_ref");
        std::printf("CASE 111: heldout_short_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 112: heldout_short_via_pptr_return_by_val
    {
        short base_val = 20;
        short* ptr = &base_val;
        short** pptr = &ptr;
        **pptr = return_h_short();
        check(**pptr == 40, "heldout_short_via_pptr_return_by_val");
        std::printf("CASE 112: heldout_short_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 113: heldout_int_local_assign
    {
        int lv = 301;
        lv = 402;
        check(lv == 402, "heldout_int_local_assign");
        std::printf("CASE 113: heldout_int_local_assign value=%d\n", (int)(lv));
    }

    // Case 114: heldout_int_local_compound_assign
    {
        int lv = 301;
        lv += 9;
        check(lv == (301) + 9, "heldout_int_local_compound_assign");
        std::printf("CASE 114: heldout_int_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 115: heldout_int_local_inc_dec
    {
        int lv = 301;
        ++lv;
        --lv;
        check(lv == 301, "heldout_int_local_inc_dec");
        std::printf("CASE 115: heldout_int_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 116: heldout_int_local_read_after_write
    {
        int lv = 301;
        lv = 402;
        int read_val = lv;
        check(read_val == 402, "heldout_int_local_read_after_write");
        std::printf("CASE 116: heldout_int_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 117: heldout_int_local_pass_by_val
    {
        int lv = 301;
        take_val_h_int(lv);
        check(lv == 301, "heldout_int_local_pass_by_val");
        std::printf("CASE 117: heldout_int_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 118: heldout_int_local_pass_by_ref
    {
        int lv = 301;
        take_ref_h_int(lv);
        check(lv == 301, "heldout_int_local_pass_by_ref");
        std::printf("CASE 118: heldout_int_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 119: heldout_int_local_return_by_val
    {
        int lv = 301;
        lv = return_h_int();
        check(lv == 402, "heldout_int_local_return_by_val");
        std::printf("CASE 119: heldout_int_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 120: heldout_int_global_assign
    {
        h_var_int = 301;
        h_var_int = 402;
        check(h_var_int == 402, "heldout_int_global_assign");
        std::printf("CASE 120: heldout_int_global_assign value=%d\n", (int)(h_var_int));
    }

    // Case 121: heldout_int_global_compound_assign
    {
        h_var_int = 301;
        h_var_int += 9;
        check(h_var_int == (301) + 9, "heldout_int_global_compound_assign");
        std::printf("CASE 121: heldout_int_global_compound_assign value=%d\n", (int)(h_var_int));
    }

    // Case 122: heldout_int_global_inc_dec
    {
        h_var_int = 301;
        ++h_var_int;
        --h_var_int;
        check(h_var_int == 301, "heldout_int_global_inc_dec");
        std::printf("CASE 122: heldout_int_global_inc_dec value=%d\n", (int)(h_var_int));
    }

    // Case 123: heldout_int_global_read_after_write
    {
        h_var_int = 301;
        h_var_int = 402;
        int read_val = h_var_int;
        check(read_val == 402, "heldout_int_global_read_after_write");
        std::printf("CASE 123: heldout_int_global_read_after_write value=%d\n", (int)(h_var_int));
    }

    // Case 124: heldout_int_global_pass_by_val
    {
        h_var_int = 301;
        take_val_h_int(h_var_int);
        check(h_var_int == 301, "heldout_int_global_pass_by_val");
        std::printf("CASE 124: heldout_int_global_pass_by_val value=%d\n", (int)(h_var_int));
    }

    // Case 125: heldout_int_global_pass_by_ref
    {
        h_var_int = 301;
        take_ref_h_int(h_var_int);
        check(h_var_int == 301, "heldout_int_global_pass_by_ref");
        std::printf("CASE 125: heldout_int_global_pass_by_ref value=%d\n", (int)(h_var_int));
    }

    // Case 126: heldout_int_global_return_by_val
    {
        h_var_int = 301;
        h_var_int = return_h_int();
        check(h_var_int == 402, "heldout_int_global_return_by_val");
        std::printf("CASE 126: heldout_int_global_return_by_val value=%d\n", (int)(h_var_int));
    }

    // Case 127: heldout_int_static_local_assign
    {
        static int s_var_int; s_var_int = 301;
        s_var_int = 402;
        check(s_var_int == 402, "heldout_int_static_local_assign");
        std::printf("CASE 127: heldout_int_static_local_assign value=%d\n", (int)(s_var_int));
    }

    // Case 128: heldout_int_static_local_compound_assign
    {
        static int s_var_int; s_var_int = 301;
        s_var_int += 9;
        check(s_var_int == (301) + 9, "heldout_int_static_local_compound_assign");
        std::printf("CASE 128: heldout_int_static_local_compound_assign value=%d\n", (int)(s_var_int));
    }

    // Case 129: heldout_int_static_local_inc_dec
    {
        static int s_var_int; s_var_int = 301;
        ++s_var_int;
        --s_var_int;
        check(s_var_int == 301, "heldout_int_static_local_inc_dec");
        std::printf("CASE 129: heldout_int_static_local_inc_dec value=%d\n", (int)(s_var_int));
    }

    // Case 130: heldout_int_static_local_read_after_write
    {
        static int s_var_int; s_var_int = 301;
        s_var_int = 402;
        int read_val = s_var_int;
        check(read_val == 402, "heldout_int_static_local_read_after_write");
        std::printf("CASE 130: heldout_int_static_local_read_after_write value=%d\n", (int)(s_var_int));
    }

    // Case 131: heldout_int_static_local_pass_by_val
    {
        static int s_var_int; s_var_int = 301;
        take_val_h_int(s_var_int);
        check(s_var_int == 301, "heldout_int_static_local_pass_by_val");
        std::printf("CASE 131: heldout_int_static_local_pass_by_val value=%d\n", (int)(s_var_int));
    }

    // Case 132: heldout_int_static_local_pass_by_ref
    {
        static int s_var_int; s_var_int = 301;
        take_ref_h_int(s_var_int);
        check(s_var_int == 301, "heldout_int_static_local_pass_by_ref");
        std::printf("CASE 132: heldout_int_static_local_pass_by_ref value=%d\n", (int)(s_var_int));
    }

    // Case 133: heldout_int_static_local_return_by_val
    {
        static int s_var_int; s_var_int = 301;
        s_var_int = return_h_int();
        check(s_var_int == 402, "heldout_int_static_local_return_by_val");
        std::printf("CASE 133: heldout_int_static_local_return_by_val value=%d\n", (int)(s_var_int));
    }

    // Case 134: heldout_int_struct_member_assign
    {
        Holder_h_int h; h.m = 301;
        h.m = 402;
        check(h.m == 402, "heldout_int_struct_member_assign");
        std::printf("CASE 134: heldout_int_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 135: heldout_int_struct_member_compound_assign
    {
        Holder_h_int h; h.m = 301;
        h.m += 9;
        check(h.m == (301) + 9, "heldout_int_struct_member_compound_assign");
        std::printf("CASE 135: heldout_int_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 136: heldout_int_struct_member_inc_dec
    {
        Holder_h_int h; h.m = 301;
        ++h.m;
        --h.m;
        check(h.m == 301, "heldout_int_struct_member_inc_dec");
        std::printf("CASE 136: heldout_int_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 137: heldout_int_struct_member_read_after_write
    {
        Holder_h_int h; h.m = 301;
        h.m = 402;
        int read_val = h.m;
        check(read_val == 402, "heldout_int_struct_member_read_after_write");
        std::printf("CASE 137: heldout_int_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 138: heldout_int_struct_member_pass_by_val
    {
        Holder_h_int h; h.m = 301;
        take_val_h_int(h.m);
        check(h.m == 301, "heldout_int_struct_member_pass_by_val");
        std::printf("CASE 138: heldout_int_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 139: heldout_int_struct_member_pass_by_ref
    {
        Holder_h_int h; h.m = 301;
        take_ref_h_int(h.m);
        check(h.m == 301, "heldout_int_struct_member_pass_by_ref");
        std::printf("CASE 139: heldout_int_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 140: heldout_int_struct_member_return_by_val
    {
        Holder_h_int h; h.m = 301;
        h.m = return_h_int();
        check(h.m == 402, "heldout_int_struct_member_return_by_val");
        std::printf("CASE 140: heldout_int_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 141: heldout_int_array_elem_assign
    {
        int arr[3] = { 301, 301, 301 };
        arr[1] = 402;
        check(arr[1] == 402, "heldout_int_array_elem_assign");
        std::printf("CASE 141: heldout_int_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 142: heldout_int_array_elem_compound_assign
    {
        int arr[3] = { 301, 301, 301 };
        arr[1] += 9;
        check(arr[1] == (301) + 9, "heldout_int_array_elem_compound_assign");
        std::printf("CASE 142: heldout_int_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 143: heldout_int_array_elem_inc_dec
    {
        int arr[3] = { 301, 301, 301 };
        ++arr[1];
        --arr[1];
        check(arr[1] == 301, "heldout_int_array_elem_inc_dec");
        std::printf("CASE 143: heldout_int_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 144: heldout_int_array_elem_read_after_write
    {
        int arr[3] = { 301, 301, 301 };
        arr[1] = 402;
        int read_val = arr[1];
        check(read_val == 402, "heldout_int_array_elem_read_after_write");
        std::printf("CASE 144: heldout_int_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 145: heldout_int_array_elem_pass_by_val
    {
        int arr[3] = { 301, 301, 301 };
        take_val_h_int(arr[1]);
        check(arr[1] == 301, "heldout_int_array_elem_pass_by_val");
        std::printf("CASE 145: heldout_int_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 146: heldout_int_array_elem_pass_by_ref
    {
        int arr[3] = { 301, 301, 301 };
        take_ref_h_int(arr[1]);
        check(arr[1] == 301, "heldout_int_array_elem_pass_by_ref");
        std::printf("CASE 146: heldout_int_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 147: heldout_int_array_elem_return_by_val
    {
        int arr[3] = { 301, 301, 301 };
        arr[1] = return_h_int();
        check(arr[1] == 402, "heldout_int_array_elem_return_by_val");
        std::printf("CASE 147: heldout_int_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 148: heldout_int_via_ref_assign
    {
        int base_val = 301;
        int& ref = base_val;
        ref = 402;
        check(ref == 402, "heldout_int_via_ref_assign");
        std::printf("CASE 148: heldout_int_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 149: heldout_int_via_ref_compound_assign
    {
        int base_val = 301;
        int& ref = base_val;
        ref += 9;
        check(ref == (301) + 9, "heldout_int_via_ref_compound_assign");
        std::printf("CASE 149: heldout_int_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 150: heldout_int_via_ref_inc_dec
    {
        int base_val = 301;
        int& ref = base_val;
        ++ref;
        --ref;
        check(ref == 301, "heldout_int_via_ref_inc_dec");
        std::printf("CASE 150: heldout_int_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 151: heldout_int_via_ref_read_after_write
    {
        int base_val = 301;
        int& ref = base_val;
        ref = 402;
        int read_val = ref;
        check(read_val == 402, "heldout_int_via_ref_read_after_write");
        std::printf("CASE 151: heldout_int_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 152: heldout_int_via_ref_pass_by_val
    {
        int base_val = 301;
        int& ref = base_val;
        take_val_h_int(ref);
        check(ref == 301, "heldout_int_via_ref_pass_by_val");
        std::printf("CASE 152: heldout_int_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 153: heldout_int_via_ref_pass_by_ref
    {
        int base_val = 301;
        int& ref = base_val;
        take_ref_h_int(ref);
        check(ref == 301, "heldout_int_via_ref_pass_by_ref");
        std::printf("CASE 153: heldout_int_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 154: heldout_int_via_ref_return_by_val
    {
        int base_val = 301;
        int& ref = base_val;
        ref = return_h_int();
        check(ref == 402, "heldout_int_via_ref_return_by_val");
        std::printf("CASE 154: heldout_int_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 155: heldout_int_via_ptr_assign
    {
        int base_val = 301;
        int* ptr = &base_val;
        *ptr = 402;
        check(*ptr == 402, "heldout_int_via_ptr_assign");
        std::printf("CASE 155: heldout_int_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 156: heldout_int_via_ptr_compound_assign
    {
        int base_val = 301;
        int* ptr = &base_val;
        *ptr += 9;
        check(*ptr == (301) + 9, "heldout_int_via_ptr_compound_assign");
        std::printf("CASE 156: heldout_int_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 157: heldout_int_via_ptr_inc_dec
    {
        int base_val = 301;
        int* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 301, "heldout_int_via_ptr_inc_dec");
        std::printf("CASE 157: heldout_int_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 158: heldout_int_via_ptr_read_after_write
    {
        int base_val = 301;
        int* ptr = &base_val;
        *ptr = 402;
        int read_val = *ptr;
        check(read_val == 402, "heldout_int_via_ptr_read_after_write");
        std::printf("CASE 158: heldout_int_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 159: heldout_int_via_ptr_pass_by_val
    {
        int base_val = 301;
        int* ptr = &base_val;
        take_val_h_int(*ptr);
        check(*ptr == 301, "heldout_int_via_ptr_pass_by_val");
        std::printf("CASE 159: heldout_int_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 160: heldout_int_via_ptr_pass_by_ref
    {
        int base_val = 301;
        int* ptr = &base_val;
        take_ref_h_int(*ptr);
        check(*ptr == 301, "heldout_int_via_ptr_pass_by_ref");
        std::printf("CASE 160: heldout_int_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 161: heldout_int_via_ptr_return_by_val
    {
        int base_val = 301;
        int* ptr = &base_val;
        *ptr = return_h_int();
        check(*ptr == 402, "heldout_int_via_ptr_return_by_val");
        std::printf("CASE 161: heldout_int_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 162: heldout_int_via_pptr_assign
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr = 402;
        check(**pptr == 402, "heldout_int_via_pptr_assign");
        std::printf("CASE 162: heldout_int_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 163: heldout_int_via_pptr_compound_assign
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr += 9;
        check(**pptr == (301) + 9, "heldout_int_via_pptr_compound_assign");
        std::printf("CASE 163: heldout_int_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 164: heldout_int_via_pptr_inc_dec
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 301, "heldout_int_via_pptr_inc_dec");
        std::printf("CASE 164: heldout_int_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 165: heldout_int_via_pptr_read_after_write
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr = 402;
        int read_val = **pptr;
        check(read_val == 402, "heldout_int_via_pptr_read_after_write");
        std::printf("CASE 165: heldout_int_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 166: heldout_int_via_pptr_pass_by_val
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        take_val_h_int(**pptr);
        check(**pptr == 301, "heldout_int_via_pptr_pass_by_val");
        std::printf("CASE 166: heldout_int_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 167: heldout_int_via_pptr_pass_by_ref
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        take_ref_h_int(**pptr);
        check(**pptr == 301, "heldout_int_via_pptr_pass_by_ref");
        std::printf("CASE 167: heldout_int_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 168: heldout_int_via_pptr_return_by_val
    {
        int base_val = 301;
        int* ptr = &base_val;
        int** pptr = &ptr;
        **pptr = return_h_int();
        check(**pptr == 402, "heldout_int_via_pptr_return_by_val");
        std::printf("CASE 168: heldout_int_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 169: heldout_long_long_local_assign
    {
        long long lv = 3000000000LL;
        lv = 4000000000LL;
        check(lv == 4000000000LL, "heldout_long_long_local_assign");
        std::printf("CASE 169: heldout_long_long_local_assign value=%d\n", (int)(lv));
    }

    // Case 170: heldout_long_long_local_compound_assign
    {
        long long lv = 3000000000LL;
        lv += 150;
        check(lv == (3000000000LL) + 150, "heldout_long_long_local_compound_assign");
        std::printf("CASE 170: heldout_long_long_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 171: heldout_long_long_local_inc_dec
    {
        long long lv = 3000000000LL;
        ++lv;
        --lv;
        check(lv == 3000000000LL, "heldout_long_long_local_inc_dec");
        std::printf("CASE 171: heldout_long_long_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 172: heldout_long_long_local_read_after_write
    {
        long long lv = 3000000000LL;
        lv = 4000000000LL;
        long long read_val = lv;
        check(read_val == 4000000000LL, "heldout_long_long_local_read_after_write");
        std::printf("CASE 172: heldout_long_long_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 173: heldout_long_long_local_pass_by_val
    {
        long long lv = 3000000000LL;
        take_val_h_long_long(lv);
        check(lv == 3000000000LL, "heldout_long_long_local_pass_by_val");
        std::printf("CASE 173: heldout_long_long_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 174: heldout_long_long_local_pass_by_ref
    {
        long long lv = 3000000000LL;
        take_ref_h_long_long(lv);
        check(lv == 3000000000LL, "heldout_long_long_local_pass_by_ref");
        std::printf("CASE 174: heldout_long_long_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 175: heldout_long_long_local_return_by_val
    {
        long long lv = 3000000000LL;
        lv = return_h_long_long();
        check(lv == 4000000000LL, "heldout_long_long_local_return_by_val");
        std::printf("CASE 175: heldout_long_long_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 176: heldout_long_long_global_assign
    {
        h_var_long_long = 3000000000LL;
        h_var_long_long = 4000000000LL;
        check(h_var_long_long == 4000000000LL, "heldout_long_long_global_assign");
        std::printf("CASE 176: heldout_long_long_global_assign value=%d\n", (int)(h_var_long_long));
    }

    // Case 177: heldout_long_long_global_compound_assign
    {
        h_var_long_long = 3000000000LL;
        h_var_long_long += 150;
        check(h_var_long_long == (3000000000LL) + 150, "heldout_long_long_global_compound_assign");
        std::printf("CASE 177: heldout_long_long_global_compound_assign value=%d\n", (int)(h_var_long_long));
    }

    // Case 178: heldout_long_long_global_inc_dec
    {
        h_var_long_long = 3000000000LL;
        ++h_var_long_long;
        --h_var_long_long;
        check(h_var_long_long == 3000000000LL, "heldout_long_long_global_inc_dec");
        std::printf("CASE 178: heldout_long_long_global_inc_dec value=%d\n", (int)(h_var_long_long));
    }

    // Case 179: heldout_long_long_global_read_after_write
    {
        h_var_long_long = 3000000000LL;
        h_var_long_long = 4000000000LL;
        long long read_val = h_var_long_long;
        check(read_val == 4000000000LL, "heldout_long_long_global_read_after_write");
        std::printf("CASE 179: heldout_long_long_global_read_after_write value=%d\n", (int)(h_var_long_long));
    }

    // Case 180: heldout_long_long_global_pass_by_val
    {
        h_var_long_long = 3000000000LL;
        take_val_h_long_long(h_var_long_long);
        check(h_var_long_long == 3000000000LL, "heldout_long_long_global_pass_by_val");
        std::printf("CASE 180: heldout_long_long_global_pass_by_val value=%d\n", (int)(h_var_long_long));
    }

    // Case 181: heldout_long_long_global_pass_by_ref
    {
        h_var_long_long = 3000000000LL;
        take_ref_h_long_long(h_var_long_long);
        check(h_var_long_long == 3000000000LL, "heldout_long_long_global_pass_by_ref");
        std::printf("CASE 181: heldout_long_long_global_pass_by_ref value=%d\n", (int)(h_var_long_long));
    }

    // Case 182: heldout_long_long_global_return_by_val
    {
        h_var_long_long = 3000000000LL;
        h_var_long_long = return_h_long_long();
        check(h_var_long_long == 4000000000LL, "heldout_long_long_global_return_by_val");
        std::printf("CASE 182: heldout_long_long_global_return_by_val value=%d\n", (int)(h_var_long_long));
    }

    // Case 183: heldout_long_long_static_local_assign
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        s_var_long_long = 4000000000LL;
        check(s_var_long_long == 4000000000LL, "heldout_long_long_static_local_assign");
        std::printf("CASE 183: heldout_long_long_static_local_assign value=%d\n", (int)(s_var_long_long));
    }

    // Case 184: heldout_long_long_static_local_compound_assign
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        s_var_long_long += 150;
        check(s_var_long_long == (3000000000LL) + 150, "heldout_long_long_static_local_compound_assign");
        std::printf("CASE 184: heldout_long_long_static_local_compound_assign value=%d\n", (int)(s_var_long_long));
    }

    // Case 185: heldout_long_long_static_local_inc_dec
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        ++s_var_long_long;
        --s_var_long_long;
        check(s_var_long_long == 3000000000LL, "heldout_long_long_static_local_inc_dec");
        std::printf("CASE 185: heldout_long_long_static_local_inc_dec value=%d\n", (int)(s_var_long_long));
    }

    // Case 186: heldout_long_long_static_local_read_after_write
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        s_var_long_long = 4000000000LL;
        long long read_val = s_var_long_long;
        check(read_val == 4000000000LL, "heldout_long_long_static_local_read_after_write");
        std::printf("CASE 186: heldout_long_long_static_local_read_after_write value=%d\n", (int)(s_var_long_long));
    }

    // Case 187: heldout_long_long_static_local_pass_by_val
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        take_val_h_long_long(s_var_long_long);
        check(s_var_long_long == 3000000000LL, "heldout_long_long_static_local_pass_by_val");
        std::printf("CASE 187: heldout_long_long_static_local_pass_by_val value=%d\n", (int)(s_var_long_long));
    }

    // Case 188: heldout_long_long_static_local_pass_by_ref
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        take_ref_h_long_long(s_var_long_long);
        check(s_var_long_long == 3000000000LL, "heldout_long_long_static_local_pass_by_ref");
        std::printf("CASE 188: heldout_long_long_static_local_pass_by_ref value=%d\n", (int)(s_var_long_long));
    }

    // Case 189: heldout_long_long_static_local_return_by_val
    {
        static long long s_var_long_long; s_var_long_long = 3000000000LL;
        s_var_long_long = return_h_long_long();
        check(s_var_long_long == 4000000000LL, "heldout_long_long_static_local_return_by_val");
        std::printf("CASE 189: heldout_long_long_static_local_return_by_val value=%d\n", (int)(s_var_long_long));
    }

    // Case 190: heldout_long_long_struct_member_assign
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        h.m = 4000000000LL;
        check(h.m == 4000000000LL, "heldout_long_long_struct_member_assign");
        std::printf("CASE 190: heldout_long_long_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 191: heldout_long_long_struct_member_compound_assign
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        h.m += 150;
        check(h.m == (3000000000LL) + 150, "heldout_long_long_struct_member_compound_assign");
        std::printf("CASE 191: heldout_long_long_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 192: heldout_long_long_struct_member_inc_dec
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        ++h.m;
        --h.m;
        check(h.m == 3000000000LL, "heldout_long_long_struct_member_inc_dec");
        std::printf("CASE 192: heldout_long_long_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 193: heldout_long_long_struct_member_read_after_write
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        h.m = 4000000000LL;
        long long read_val = h.m;
        check(read_val == 4000000000LL, "heldout_long_long_struct_member_read_after_write");
        std::printf("CASE 193: heldout_long_long_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 194: heldout_long_long_struct_member_pass_by_val
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        take_val_h_long_long(h.m);
        check(h.m == 3000000000LL, "heldout_long_long_struct_member_pass_by_val");
        std::printf("CASE 194: heldout_long_long_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 195: heldout_long_long_struct_member_pass_by_ref
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        take_ref_h_long_long(h.m);
        check(h.m == 3000000000LL, "heldout_long_long_struct_member_pass_by_ref");
        std::printf("CASE 195: heldout_long_long_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 196: heldout_long_long_struct_member_return_by_val
    {
        Holder_h_long_long h; h.m = 3000000000LL;
        h.m = return_h_long_long();
        check(h.m == 4000000000LL, "heldout_long_long_struct_member_return_by_val");
        std::printf("CASE 196: heldout_long_long_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 197: heldout_long_long_array_elem_assign
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        arr[1] = 4000000000LL;
        check(arr[1] == 4000000000LL, "heldout_long_long_array_elem_assign");
        std::printf("CASE 197: heldout_long_long_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 198: heldout_long_long_array_elem_compound_assign
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        arr[1] += 150;
        check(arr[1] == (3000000000LL) + 150, "heldout_long_long_array_elem_compound_assign");
        std::printf("CASE 198: heldout_long_long_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 199: heldout_long_long_array_elem_inc_dec
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        ++arr[1];
        --arr[1];
        check(arr[1] == 3000000000LL, "heldout_long_long_array_elem_inc_dec");
        std::printf("CASE 199: heldout_long_long_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 200: heldout_long_long_array_elem_read_after_write
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        arr[1] = 4000000000LL;
        long long read_val = arr[1];
        check(read_val == 4000000000LL, "heldout_long_long_array_elem_read_after_write");
        std::printf("CASE 200: heldout_long_long_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 201: heldout_long_long_array_elem_pass_by_val
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        take_val_h_long_long(arr[1]);
        check(arr[1] == 3000000000LL, "heldout_long_long_array_elem_pass_by_val");
        std::printf("CASE 201: heldout_long_long_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 202: heldout_long_long_array_elem_pass_by_ref
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        take_ref_h_long_long(arr[1]);
        check(arr[1] == 3000000000LL, "heldout_long_long_array_elem_pass_by_ref");
        std::printf("CASE 202: heldout_long_long_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 203: heldout_long_long_array_elem_return_by_val
    {
        long long arr[3] = { 3000000000LL, 3000000000LL, 3000000000LL };
        arr[1] = return_h_long_long();
        check(arr[1] == 4000000000LL, "heldout_long_long_array_elem_return_by_val");
        std::printf("CASE 203: heldout_long_long_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 204: heldout_long_long_via_ref_assign
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        ref = 4000000000LL;
        check(ref == 4000000000LL, "heldout_long_long_via_ref_assign");
        std::printf("CASE 204: heldout_long_long_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 205: heldout_long_long_via_ref_compound_assign
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        ref += 150;
        check(ref == (3000000000LL) + 150, "heldout_long_long_via_ref_compound_assign");
        std::printf("CASE 205: heldout_long_long_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 206: heldout_long_long_via_ref_inc_dec
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        ++ref;
        --ref;
        check(ref == 3000000000LL, "heldout_long_long_via_ref_inc_dec");
        std::printf("CASE 206: heldout_long_long_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 207: heldout_long_long_via_ref_read_after_write
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        ref = 4000000000LL;
        long long read_val = ref;
        check(read_val == 4000000000LL, "heldout_long_long_via_ref_read_after_write");
        std::printf("CASE 207: heldout_long_long_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 208: heldout_long_long_via_ref_pass_by_val
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        take_val_h_long_long(ref);
        check(ref == 3000000000LL, "heldout_long_long_via_ref_pass_by_val");
        std::printf("CASE 208: heldout_long_long_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 209: heldout_long_long_via_ref_pass_by_ref
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        take_ref_h_long_long(ref);
        check(ref == 3000000000LL, "heldout_long_long_via_ref_pass_by_ref");
        std::printf("CASE 209: heldout_long_long_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 210: heldout_long_long_via_ref_return_by_val
    {
        long long base_val = 3000000000LL;
        long long& ref = base_val;
        ref = return_h_long_long();
        check(ref == 4000000000LL, "heldout_long_long_via_ref_return_by_val");
        std::printf("CASE 210: heldout_long_long_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 211: heldout_long_long_via_ptr_assign
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        *ptr = 4000000000LL;
        check(*ptr == 4000000000LL, "heldout_long_long_via_ptr_assign");
        std::printf("CASE 211: heldout_long_long_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 212: heldout_long_long_via_ptr_compound_assign
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        *ptr += 150;
        check(*ptr == (3000000000LL) + 150, "heldout_long_long_via_ptr_compound_assign");
        std::printf("CASE 212: heldout_long_long_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 213: heldout_long_long_via_ptr_inc_dec
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 3000000000LL, "heldout_long_long_via_ptr_inc_dec");
        std::printf("CASE 213: heldout_long_long_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 214: heldout_long_long_via_ptr_read_after_write
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        *ptr = 4000000000LL;
        long long read_val = *ptr;
        check(read_val == 4000000000LL, "heldout_long_long_via_ptr_read_after_write");
        std::printf("CASE 214: heldout_long_long_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 215: heldout_long_long_via_ptr_pass_by_val
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        take_val_h_long_long(*ptr);
        check(*ptr == 3000000000LL, "heldout_long_long_via_ptr_pass_by_val");
        std::printf("CASE 215: heldout_long_long_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 216: heldout_long_long_via_ptr_pass_by_ref
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        take_ref_h_long_long(*ptr);
        check(*ptr == 3000000000LL, "heldout_long_long_via_ptr_pass_by_ref");
        std::printf("CASE 216: heldout_long_long_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 217: heldout_long_long_via_ptr_return_by_val
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        *ptr = return_h_long_long();
        check(*ptr == 4000000000LL, "heldout_long_long_via_ptr_return_by_val");
        std::printf("CASE 217: heldout_long_long_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 218: heldout_long_long_via_pptr_assign
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr = 4000000000LL;
        check(**pptr == 4000000000LL, "heldout_long_long_via_pptr_assign");
        std::printf("CASE 218: heldout_long_long_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 219: heldout_long_long_via_pptr_compound_assign
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr += 150;
        check(**pptr == (3000000000LL) + 150, "heldout_long_long_via_pptr_compound_assign");
        std::printf("CASE 219: heldout_long_long_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 220: heldout_long_long_via_pptr_inc_dec
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 3000000000LL, "heldout_long_long_via_pptr_inc_dec");
        std::printf("CASE 220: heldout_long_long_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 221: heldout_long_long_via_pptr_read_after_write
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr = 4000000000LL;
        long long read_val = **pptr;
        check(read_val == 4000000000LL, "heldout_long_long_via_pptr_read_after_write");
        std::printf("CASE 221: heldout_long_long_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 222: heldout_long_long_via_pptr_pass_by_val
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        take_val_h_long_long(**pptr);
        check(**pptr == 3000000000LL, "heldout_long_long_via_pptr_pass_by_val");
        std::printf("CASE 222: heldout_long_long_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 223: heldout_long_long_via_pptr_pass_by_ref
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        take_ref_h_long_long(**pptr);
        check(**pptr == 3000000000LL, "heldout_long_long_via_pptr_pass_by_ref");
        std::printf("CASE 223: heldout_long_long_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 224: heldout_long_long_via_pptr_return_by_val
    {
        long long base_val = 3000000000LL;
        long long* ptr = &base_val;
        long long** pptr = &ptr;
        **pptr = return_h_long_long();
        check(**pptr == 4000000000LL, "heldout_long_long_via_pptr_return_by_val");
        std::printf("CASE 224: heldout_long_long_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 225: heldout_float_local_assign
    {
        float lv = 2.5f;
        lv = 6.5f;
        check(lv == 6.5f, "heldout_float_local_assign");
        std::printf("CASE 225: heldout_float_local_assign value=%d\n", (int)(lv));
    }

    // Case 226: heldout_float_local_compound_assign
    {
        float lv = 2.5f;
        lv += 3.0;
        check(lv == (2.5f) + 3.0, "heldout_float_local_compound_assign");
        std::printf("CASE 226: heldout_float_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 227: heldout_float_local_inc_dec
    {
        float lv = 2.5f;
        ++lv;
        --lv;
        check(lv == 2.5f, "heldout_float_local_inc_dec");
        std::printf("CASE 227: heldout_float_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 228: heldout_float_local_read_after_write
    {
        float lv = 2.5f;
        lv = 6.5f;
        float read_val = lv;
        check(read_val == 6.5f, "heldout_float_local_read_after_write");
        std::printf("CASE 228: heldout_float_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 229: heldout_float_local_pass_by_val
    {
        float lv = 2.5f;
        take_val_h_float(lv);
        check(lv == 2.5f, "heldout_float_local_pass_by_val");
        std::printf("CASE 229: heldout_float_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 230: heldout_float_local_pass_by_ref
    {
        float lv = 2.5f;
        take_ref_h_float(lv);
        check(lv == 2.5f, "heldout_float_local_pass_by_ref");
        std::printf("CASE 230: heldout_float_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 231: heldout_float_local_return_by_val
    {
        float lv = 2.5f;
        lv = return_h_float();
        check(lv == 6.5f, "heldout_float_local_return_by_val");
        std::printf("CASE 231: heldout_float_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 232: heldout_float_global_assign
    {
        h_var_float = 2.5f;
        h_var_float = 6.5f;
        check(h_var_float == 6.5f, "heldout_float_global_assign");
        std::printf("CASE 232: heldout_float_global_assign value=%d\n", (int)(h_var_float));
    }

    // Case 233: heldout_float_global_compound_assign
    {
        h_var_float = 2.5f;
        h_var_float += 3.0;
        check(h_var_float == (2.5f) + 3.0, "heldout_float_global_compound_assign");
        std::printf("CASE 233: heldout_float_global_compound_assign value=%d\n", (int)(h_var_float));
    }

    // Case 234: heldout_float_global_inc_dec
    {
        h_var_float = 2.5f;
        ++h_var_float;
        --h_var_float;
        check(h_var_float == 2.5f, "heldout_float_global_inc_dec");
        std::printf("CASE 234: heldout_float_global_inc_dec value=%d\n", (int)(h_var_float));
    }

    // Case 235: heldout_float_global_read_after_write
    {
        h_var_float = 2.5f;
        h_var_float = 6.5f;
        float read_val = h_var_float;
        check(read_val == 6.5f, "heldout_float_global_read_after_write");
        std::printf("CASE 235: heldout_float_global_read_after_write value=%d\n", (int)(h_var_float));
    }

    // Case 236: heldout_float_global_pass_by_val
    {
        h_var_float = 2.5f;
        take_val_h_float(h_var_float);
        check(h_var_float == 2.5f, "heldout_float_global_pass_by_val");
        std::printf("CASE 236: heldout_float_global_pass_by_val value=%d\n", (int)(h_var_float));
    }

    // Case 237: heldout_float_global_pass_by_ref
    {
        h_var_float = 2.5f;
        take_ref_h_float(h_var_float);
        check(h_var_float == 2.5f, "heldout_float_global_pass_by_ref");
        std::printf("CASE 237: heldout_float_global_pass_by_ref value=%d\n", (int)(h_var_float));
    }

    // Case 238: heldout_float_global_return_by_val
    {
        h_var_float = 2.5f;
        h_var_float = return_h_float();
        check(h_var_float == 6.5f, "heldout_float_global_return_by_val");
        std::printf("CASE 238: heldout_float_global_return_by_val value=%d\n", (int)(h_var_float));
    }

    // Case 239: heldout_float_static_local_assign
    {
        static float s_var_float; s_var_float = 2.5f;
        s_var_float = 6.5f;
        check(s_var_float == 6.5f, "heldout_float_static_local_assign");
        std::printf("CASE 239: heldout_float_static_local_assign value=%d\n", (int)(s_var_float));
    }

    // Case 240: heldout_float_static_local_compound_assign
    {
        static float s_var_float; s_var_float = 2.5f;
        s_var_float += 3.0;
        check(s_var_float == (2.5f) + 3.0, "heldout_float_static_local_compound_assign");
        std::printf("CASE 240: heldout_float_static_local_compound_assign value=%d\n", (int)(s_var_float));
    }

    // Case 241: heldout_float_static_local_inc_dec
    {
        static float s_var_float; s_var_float = 2.5f;
        ++s_var_float;
        --s_var_float;
        check(s_var_float == 2.5f, "heldout_float_static_local_inc_dec");
        std::printf("CASE 241: heldout_float_static_local_inc_dec value=%d\n", (int)(s_var_float));
    }

    // Case 242: heldout_float_static_local_read_after_write
    {
        static float s_var_float; s_var_float = 2.5f;
        s_var_float = 6.5f;
        float read_val = s_var_float;
        check(read_val == 6.5f, "heldout_float_static_local_read_after_write");
        std::printf("CASE 242: heldout_float_static_local_read_after_write value=%d\n", (int)(s_var_float));
    }

    // Case 243: heldout_float_static_local_pass_by_val
    {
        static float s_var_float; s_var_float = 2.5f;
        take_val_h_float(s_var_float);
        check(s_var_float == 2.5f, "heldout_float_static_local_pass_by_val");
        std::printf("CASE 243: heldout_float_static_local_pass_by_val value=%d\n", (int)(s_var_float));
    }

    // Case 244: heldout_float_static_local_pass_by_ref
    {
        static float s_var_float; s_var_float = 2.5f;
        take_ref_h_float(s_var_float);
        check(s_var_float == 2.5f, "heldout_float_static_local_pass_by_ref");
        std::printf("CASE 244: heldout_float_static_local_pass_by_ref value=%d\n", (int)(s_var_float));
    }

    // Case 245: heldout_float_static_local_return_by_val
    {
        static float s_var_float; s_var_float = 2.5f;
        s_var_float = return_h_float();
        check(s_var_float == 6.5f, "heldout_float_static_local_return_by_val");
        std::printf("CASE 245: heldout_float_static_local_return_by_val value=%d\n", (int)(s_var_float));
    }

    // Case 246: heldout_float_struct_member_assign
    {
        Holder_h_float h; h.m = 2.5f;
        h.m = 6.5f;
        check(h.m == 6.5f, "heldout_float_struct_member_assign");
        std::printf("CASE 246: heldout_float_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 247: heldout_float_struct_member_compound_assign
    {
        Holder_h_float h; h.m = 2.5f;
        h.m += 3.0;
        check(h.m == (2.5f) + 3.0, "heldout_float_struct_member_compound_assign");
        std::printf("CASE 247: heldout_float_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 248: heldout_float_struct_member_inc_dec
    {
        Holder_h_float h; h.m = 2.5f;
        ++h.m;
        --h.m;
        check(h.m == 2.5f, "heldout_float_struct_member_inc_dec");
        std::printf("CASE 248: heldout_float_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 249: heldout_float_struct_member_read_after_write
    {
        Holder_h_float h; h.m = 2.5f;
        h.m = 6.5f;
        float read_val = h.m;
        check(read_val == 6.5f, "heldout_float_struct_member_read_after_write");
        std::printf("CASE 249: heldout_float_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 250: heldout_float_struct_member_pass_by_val
    {
        Holder_h_float h; h.m = 2.5f;
        take_val_h_float(h.m);
        check(h.m == 2.5f, "heldout_float_struct_member_pass_by_val");
        std::printf("CASE 250: heldout_float_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 251: heldout_float_struct_member_pass_by_ref
    {
        Holder_h_float h; h.m = 2.5f;
        take_ref_h_float(h.m);
        check(h.m == 2.5f, "heldout_float_struct_member_pass_by_ref");
        std::printf("CASE 251: heldout_float_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 252: heldout_float_struct_member_return_by_val
    {
        Holder_h_float h; h.m = 2.5f;
        h.m = return_h_float();
        check(h.m == 6.5f, "heldout_float_struct_member_return_by_val");
        std::printf("CASE 252: heldout_float_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 253: heldout_float_array_elem_assign
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        arr[1] = 6.5f;
        check(arr[1] == 6.5f, "heldout_float_array_elem_assign");
        std::printf("CASE 253: heldout_float_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 254: heldout_float_array_elem_compound_assign
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        arr[1] += 3.0;
        check(arr[1] == (2.5f) + 3.0, "heldout_float_array_elem_compound_assign");
        std::printf("CASE 254: heldout_float_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 255: heldout_float_array_elem_inc_dec
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        ++arr[1];
        --arr[1];
        check(arr[1] == 2.5f, "heldout_float_array_elem_inc_dec");
        std::printf("CASE 255: heldout_float_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 256: heldout_float_array_elem_read_after_write
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        arr[1] = 6.5f;
        float read_val = arr[1];
        check(read_val == 6.5f, "heldout_float_array_elem_read_after_write");
        std::printf("CASE 256: heldout_float_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 257: heldout_float_array_elem_pass_by_val
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        take_val_h_float(arr[1]);
        check(arr[1] == 2.5f, "heldout_float_array_elem_pass_by_val");
        std::printf("CASE 257: heldout_float_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 258: heldout_float_array_elem_pass_by_ref
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        take_ref_h_float(arr[1]);
        check(arr[1] == 2.5f, "heldout_float_array_elem_pass_by_ref");
        std::printf("CASE 258: heldout_float_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 259: heldout_float_array_elem_return_by_val
    {
        float arr[3] = { 2.5f, 2.5f, 2.5f };
        arr[1] = return_h_float();
        check(arr[1] == 6.5f, "heldout_float_array_elem_return_by_val");
        std::printf("CASE 259: heldout_float_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 260: heldout_float_via_ref_assign
    {
        float base_val = 2.5f;
        float& ref = base_val;
        ref = 6.5f;
        check(ref == 6.5f, "heldout_float_via_ref_assign");
        std::printf("CASE 260: heldout_float_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 261: heldout_float_via_ref_compound_assign
    {
        float base_val = 2.5f;
        float& ref = base_val;
        ref += 3.0;
        check(ref == (2.5f) + 3.0, "heldout_float_via_ref_compound_assign");
        std::printf("CASE 261: heldout_float_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 262: heldout_float_via_ref_inc_dec
    {
        float base_val = 2.5f;
        float& ref = base_val;
        ++ref;
        --ref;
        check(ref == 2.5f, "heldout_float_via_ref_inc_dec");
        std::printf("CASE 262: heldout_float_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 263: heldout_float_via_ref_read_after_write
    {
        float base_val = 2.5f;
        float& ref = base_val;
        ref = 6.5f;
        float read_val = ref;
        check(read_val == 6.5f, "heldout_float_via_ref_read_after_write");
        std::printf("CASE 263: heldout_float_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 264: heldout_float_via_ref_pass_by_val
    {
        float base_val = 2.5f;
        float& ref = base_val;
        take_val_h_float(ref);
        check(ref == 2.5f, "heldout_float_via_ref_pass_by_val");
        std::printf("CASE 264: heldout_float_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 265: heldout_float_via_ref_pass_by_ref
    {
        float base_val = 2.5f;
        float& ref = base_val;
        take_ref_h_float(ref);
        check(ref == 2.5f, "heldout_float_via_ref_pass_by_ref");
        std::printf("CASE 265: heldout_float_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 266: heldout_float_via_ref_return_by_val
    {
        float base_val = 2.5f;
        float& ref = base_val;
        ref = return_h_float();
        check(ref == 6.5f, "heldout_float_via_ref_return_by_val");
        std::printf("CASE 266: heldout_float_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 267: heldout_float_via_ptr_assign
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        *ptr = 6.5f;
        check(*ptr == 6.5f, "heldout_float_via_ptr_assign");
        std::printf("CASE 267: heldout_float_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 268: heldout_float_via_ptr_compound_assign
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        *ptr += 3.0;
        check(*ptr == (2.5f) + 3.0, "heldout_float_via_ptr_compound_assign");
        std::printf("CASE 268: heldout_float_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 269: heldout_float_via_ptr_inc_dec
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 2.5f, "heldout_float_via_ptr_inc_dec");
        std::printf("CASE 269: heldout_float_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 270: heldout_float_via_ptr_read_after_write
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        *ptr = 6.5f;
        float read_val = *ptr;
        check(read_val == 6.5f, "heldout_float_via_ptr_read_after_write");
        std::printf("CASE 270: heldout_float_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 271: heldout_float_via_ptr_pass_by_val
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        take_val_h_float(*ptr);
        check(*ptr == 2.5f, "heldout_float_via_ptr_pass_by_val");
        std::printf("CASE 271: heldout_float_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 272: heldout_float_via_ptr_pass_by_ref
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        take_ref_h_float(*ptr);
        check(*ptr == 2.5f, "heldout_float_via_ptr_pass_by_ref");
        std::printf("CASE 272: heldout_float_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 273: heldout_float_via_ptr_return_by_val
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        *ptr = return_h_float();
        check(*ptr == 6.5f, "heldout_float_via_ptr_return_by_val");
        std::printf("CASE 273: heldout_float_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 274: heldout_float_via_pptr_assign
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr = 6.5f;
        check(**pptr == 6.5f, "heldout_float_via_pptr_assign");
        std::printf("CASE 274: heldout_float_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 275: heldout_float_via_pptr_compound_assign
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr += 3.0;
        check(**pptr == (2.5f) + 3.0, "heldout_float_via_pptr_compound_assign");
        std::printf("CASE 275: heldout_float_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 276: heldout_float_via_pptr_inc_dec
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 2.5f, "heldout_float_via_pptr_inc_dec");
        std::printf("CASE 276: heldout_float_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 277: heldout_float_via_pptr_read_after_write
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr = 6.5f;
        float read_val = **pptr;
        check(read_val == 6.5f, "heldout_float_via_pptr_read_after_write");
        std::printf("CASE 277: heldout_float_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 278: heldout_float_via_pptr_pass_by_val
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        take_val_h_float(**pptr);
        check(**pptr == 2.5f, "heldout_float_via_pptr_pass_by_val");
        std::printf("CASE 278: heldout_float_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 279: heldout_float_via_pptr_pass_by_ref
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        take_ref_h_float(**pptr);
        check(**pptr == 2.5f, "heldout_float_via_pptr_pass_by_ref");
        std::printf("CASE 279: heldout_float_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 280: heldout_float_via_pptr_return_by_val
    {
        float base_val = 2.5f;
        float* ptr = &base_val;
        float** pptr = &ptr;
        **pptr = return_h_float();
        check(**pptr == 6.5f, "heldout_float_via_pptr_return_by_val");
        std::printf("CASE 280: heldout_float_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 281: heldout_double_local_assign
    {
        double lv = 3.25;
        lv = 7.75;
        check(lv == 7.75, "heldout_double_local_assign");
        std::printf("CASE 281: heldout_double_local_assign value=%d\n", (int)(lv));
    }

    // Case 282: heldout_double_local_compound_assign
    {
        double lv = 3.25;
        lv += 4.0;
        check(lv == (3.25) + 4.0, "heldout_double_local_compound_assign");
        std::printf("CASE 282: heldout_double_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 283: heldout_double_local_inc_dec
    {
        double lv = 3.25;
        ++lv;
        --lv;
        check(lv == 3.25, "heldout_double_local_inc_dec");
        std::printf("CASE 283: heldout_double_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 284: heldout_double_local_read_after_write
    {
        double lv = 3.25;
        lv = 7.75;
        double read_val = lv;
        check(read_val == 7.75, "heldout_double_local_read_after_write");
        std::printf("CASE 284: heldout_double_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 285: heldout_double_local_pass_by_val
    {
        double lv = 3.25;
        take_val_h_double(lv);
        check(lv == 3.25, "heldout_double_local_pass_by_val");
        std::printf("CASE 285: heldout_double_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 286: heldout_double_local_pass_by_ref
    {
        double lv = 3.25;
        take_ref_h_double(lv);
        check(lv == 3.25, "heldout_double_local_pass_by_ref");
        std::printf("CASE 286: heldout_double_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 287: heldout_double_local_return_by_val
    {
        double lv = 3.25;
        lv = return_h_double();
        check(lv == 7.75, "heldout_double_local_return_by_val");
        std::printf("CASE 287: heldout_double_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 288: heldout_double_global_assign
    {
        h_var_double = 3.25;
        h_var_double = 7.75;
        check(h_var_double == 7.75, "heldout_double_global_assign");
        std::printf("CASE 288: heldout_double_global_assign value=%d\n", (int)(h_var_double));
    }

    // Case 289: heldout_double_global_compound_assign
    {
        h_var_double = 3.25;
        h_var_double += 4.0;
        check(h_var_double == (3.25) + 4.0, "heldout_double_global_compound_assign");
        std::printf("CASE 289: heldout_double_global_compound_assign value=%d\n", (int)(h_var_double));
    }

    // Case 290: heldout_double_global_inc_dec
    {
        h_var_double = 3.25;
        ++h_var_double;
        --h_var_double;
        check(h_var_double == 3.25, "heldout_double_global_inc_dec");
        std::printf("CASE 290: heldout_double_global_inc_dec value=%d\n", (int)(h_var_double));
    }

    // Case 291: heldout_double_global_read_after_write
    {
        h_var_double = 3.25;
        h_var_double = 7.75;
        double read_val = h_var_double;
        check(read_val == 7.75, "heldout_double_global_read_after_write");
        std::printf("CASE 291: heldout_double_global_read_after_write value=%d\n", (int)(h_var_double));
    }

    // Case 292: heldout_double_global_pass_by_val
    {
        h_var_double = 3.25;
        take_val_h_double(h_var_double);
        check(h_var_double == 3.25, "heldout_double_global_pass_by_val");
        std::printf("CASE 292: heldout_double_global_pass_by_val value=%d\n", (int)(h_var_double));
    }

    // Case 293: heldout_double_global_pass_by_ref
    {
        h_var_double = 3.25;
        take_ref_h_double(h_var_double);
        check(h_var_double == 3.25, "heldout_double_global_pass_by_ref");
        std::printf("CASE 293: heldout_double_global_pass_by_ref value=%d\n", (int)(h_var_double));
    }

    // Case 294: heldout_double_global_return_by_val
    {
        h_var_double = 3.25;
        h_var_double = return_h_double();
        check(h_var_double == 7.75, "heldout_double_global_return_by_val");
        std::printf("CASE 294: heldout_double_global_return_by_val value=%d\n", (int)(h_var_double));
    }

    // Case 295: heldout_double_static_local_assign
    {
        static double s_var_double; s_var_double = 3.25;
        s_var_double = 7.75;
        check(s_var_double == 7.75, "heldout_double_static_local_assign");
        std::printf("CASE 295: heldout_double_static_local_assign value=%d\n", (int)(s_var_double));
    }

    // Case 296: heldout_double_static_local_compound_assign
    {
        static double s_var_double; s_var_double = 3.25;
        s_var_double += 4.0;
        check(s_var_double == (3.25) + 4.0, "heldout_double_static_local_compound_assign");
        std::printf("CASE 296: heldout_double_static_local_compound_assign value=%d\n", (int)(s_var_double));
    }

    // Case 297: heldout_double_static_local_inc_dec
    {
        static double s_var_double; s_var_double = 3.25;
        ++s_var_double;
        --s_var_double;
        check(s_var_double == 3.25, "heldout_double_static_local_inc_dec");
        std::printf("CASE 297: heldout_double_static_local_inc_dec value=%d\n", (int)(s_var_double));
    }

    // Case 298: heldout_double_static_local_read_after_write
    {
        static double s_var_double; s_var_double = 3.25;
        s_var_double = 7.75;
        double read_val = s_var_double;
        check(read_val == 7.75, "heldout_double_static_local_read_after_write");
        std::printf("CASE 298: heldout_double_static_local_read_after_write value=%d\n", (int)(s_var_double));
    }

    // Case 299: heldout_double_static_local_pass_by_val
    {
        static double s_var_double; s_var_double = 3.25;
        take_val_h_double(s_var_double);
        check(s_var_double == 3.25, "heldout_double_static_local_pass_by_val");
        std::printf("CASE 299: heldout_double_static_local_pass_by_val value=%d\n", (int)(s_var_double));
    }

    // Case 300: heldout_double_static_local_pass_by_ref
    {
        static double s_var_double; s_var_double = 3.25;
        take_ref_h_double(s_var_double);
        check(s_var_double == 3.25, "heldout_double_static_local_pass_by_ref");
        std::printf("CASE 300: heldout_double_static_local_pass_by_ref value=%d\n", (int)(s_var_double));
    }

    // Case 301: heldout_double_static_local_return_by_val
    {
        static double s_var_double; s_var_double = 3.25;
        s_var_double = return_h_double();
        check(s_var_double == 7.75, "heldout_double_static_local_return_by_val");
        std::printf("CASE 301: heldout_double_static_local_return_by_val value=%d\n", (int)(s_var_double));
    }

    // Case 302: heldout_double_struct_member_assign
    {
        Holder_h_double h; h.m = 3.25;
        h.m = 7.75;
        check(h.m == 7.75, "heldout_double_struct_member_assign");
        std::printf("CASE 302: heldout_double_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 303: heldout_double_struct_member_compound_assign
    {
        Holder_h_double h; h.m = 3.25;
        h.m += 4.0;
        check(h.m == (3.25) + 4.0, "heldout_double_struct_member_compound_assign");
        std::printf("CASE 303: heldout_double_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 304: heldout_double_struct_member_inc_dec
    {
        Holder_h_double h; h.m = 3.25;
        ++h.m;
        --h.m;
        check(h.m == 3.25, "heldout_double_struct_member_inc_dec");
        std::printf("CASE 304: heldout_double_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 305: heldout_double_struct_member_read_after_write
    {
        Holder_h_double h; h.m = 3.25;
        h.m = 7.75;
        double read_val = h.m;
        check(read_val == 7.75, "heldout_double_struct_member_read_after_write");
        std::printf("CASE 305: heldout_double_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 306: heldout_double_struct_member_pass_by_val
    {
        Holder_h_double h; h.m = 3.25;
        take_val_h_double(h.m);
        check(h.m == 3.25, "heldout_double_struct_member_pass_by_val");
        std::printf("CASE 306: heldout_double_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 307: heldout_double_struct_member_pass_by_ref
    {
        Holder_h_double h; h.m = 3.25;
        take_ref_h_double(h.m);
        check(h.m == 3.25, "heldout_double_struct_member_pass_by_ref");
        std::printf("CASE 307: heldout_double_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 308: heldout_double_struct_member_return_by_val
    {
        Holder_h_double h; h.m = 3.25;
        h.m = return_h_double();
        check(h.m == 7.75, "heldout_double_struct_member_return_by_val");
        std::printf("CASE 308: heldout_double_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 309: heldout_double_array_elem_assign
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        arr[1] = 7.75;
        check(arr[1] == 7.75, "heldout_double_array_elem_assign");
        std::printf("CASE 309: heldout_double_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 310: heldout_double_array_elem_compound_assign
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        arr[1] += 4.0;
        check(arr[1] == (3.25) + 4.0, "heldout_double_array_elem_compound_assign");
        std::printf("CASE 310: heldout_double_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 311: heldout_double_array_elem_inc_dec
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        ++arr[1];
        --arr[1];
        check(arr[1] == 3.25, "heldout_double_array_elem_inc_dec");
        std::printf("CASE 311: heldout_double_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 312: heldout_double_array_elem_read_after_write
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        arr[1] = 7.75;
        double read_val = arr[1];
        check(read_val == 7.75, "heldout_double_array_elem_read_after_write");
        std::printf("CASE 312: heldout_double_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 313: heldout_double_array_elem_pass_by_val
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        take_val_h_double(arr[1]);
        check(arr[1] == 3.25, "heldout_double_array_elem_pass_by_val");
        std::printf("CASE 313: heldout_double_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 314: heldout_double_array_elem_pass_by_ref
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        take_ref_h_double(arr[1]);
        check(arr[1] == 3.25, "heldout_double_array_elem_pass_by_ref");
        std::printf("CASE 314: heldout_double_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 315: heldout_double_array_elem_return_by_val
    {
        double arr[3] = { 3.25, 3.25, 3.25 };
        arr[1] = return_h_double();
        check(arr[1] == 7.75, "heldout_double_array_elem_return_by_val");
        std::printf("CASE 315: heldout_double_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 316: heldout_double_via_ref_assign
    {
        double base_val = 3.25;
        double& ref = base_val;
        ref = 7.75;
        check(ref == 7.75, "heldout_double_via_ref_assign");
        std::printf("CASE 316: heldout_double_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 317: heldout_double_via_ref_compound_assign
    {
        double base_val = 3.25;
        double& ref = base_val;
        ref += 4.0;
        check(ref == (3.25) + 4.0, "heldout_double_via_ref_compound_assign");
        std::printf("CASE 317: heldout_double_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 318: heldout_double_via_ref_inc_dec
    {
        double base_val = 3.25;
        double& ref = base_val;
        ++ref;
        --ref;
        check(ref == 3.25, "heldout_double_via_ref_inc_dec");
        std::printf("CASE 318: heldout_double_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 319: heldout_double_via_ref_read_after_write
    {
        double base_val = 3.25;
        double& ref = base_val;
        ref = 7.75;
        double read_val = ref;
        check(read_val == 7.75, "heldout_double_via_ref_read_after_write");
        std::printf("CASE 319: heldout_double_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 320: heldout_double_via_ref_pass_by_val
    {
        double base_val = 3.25;
        double& ref = base_val;
        take_val_h_double(ref);
        check(ref == 3.25, "heldout_double_via_ref_pass_by_val");
        std::printf("CASE 320: heldout_double_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 321: heldout_double_via_ref_pass_by_ref
    {
        double base_val = 3.25;
        double& ref = base_val;
        take_ref_h_double(ref);
        check(ref == 3.25, "heldout_double_via_ref_pass_by_ref");
        std::printf("CASE 321: heldout_double_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 322: heldout_double_via_ref_return_by_val
    {
        double base_val = 3.25;
        double& ref = base_val;
        ref = return_h_double();
        check(ref == 7.75, "heldout_double_via_ref_return_by_val");
        std::printf("CASE 322: heldout_double_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 323: heldout_double_via_ptr_assign
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        *ptr = 7.75;
        check(*ptr == 7.75, "heldout_double_via_ptr_assign");
        std::printf("CASE 323: heldout_double_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 324: heldout_double_via_ptr_compound_assign
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        *ptr += 4.0;
        check(*ptr == (3.25) + 4.0, "heldout_double_via_ptr_compound_assign");
        std::printf("CASE 324: heldout_double_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 325: heldout_double_via_ptr_inc_dec
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == 3.25, "heldout_double_via_ptr_inc_dec");
        std::printf("CASE 325: heldout_double_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 326: heldout_double_via_ptr_read_after_write
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        *ptr = 7.75;
        double read_val = *ptr;
        check(read_val == 7.75, "heldout_double_via_ptr_read_after_write");
        std::printf("CASE 326: heldout_double_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 327: heldout_double_via_ptr_pass_by_val
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        take_val_h_double(*ptr);
        check(*ptr == 3.25, "heldout_double_via_ptr_pass_by_val");
        std::printf("CASE 327: heldout_double_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 328: heldout_double_via_ptr_pass_by_ref
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        take_ref_h_double(*ptr);
        check(*ptr == 3.25, "heldout_double_via_ptr_pass_by_ref");
        std::printf("CASE 328: heldout_double_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 329: heldout_double_via_ptr_return_by_val
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        *ptr = return_h_double();
        check(*ptr == 7.75, "heldout_double_via_ptr_return_by_val");
        std::printf("CASE 329: heldout_double_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 330: heldout_double_via_pptr_assign
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr = 7.75;
        check(**pptr == 7.75, "heldout_double_via_pptr_assign");
        std::printf("CASE 330: heldout_double_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 331: heldout_double_via_pptr_compound_assign
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr += 4.0;
        check(**pptr == (3.25) + 4.0, "heldout_double_via_pptr_compound_assign");
        std::printf("CASE 331: heldout_double_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 332: heldout_double_via_pptr_inc_dec
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == 3.25, "heldout_double_via_pptr_inc_dec");
        std::printf("CASE 332: heldout_double_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 333: heldout_double_via_pptr_read_after_write
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr = 7.75;
        double read_val = **pptr;
        check(read_val == 7.75, "heldout_double_via_pptr_read_after_write");
        std::printf("CASE 333: heldout_double_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 334: heldout_double_via_pptr_pass_by_val
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        take_val_h_double(**pptr);
        check(**pptr == 3.25, "heldout_double_via_pptr_pass_by_val");
        std::printf("CASE 334: heldout_double_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 335: heldout_double_via_pptr_pass_by_ref
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        take_ref_h_double(**pptr);
        check(**pptr == 3.25, "heldout_double_via_pptr_pass_by_ref");
        std::printf("CASE 335: heldout_double_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 336: heldout_double_via_pptr_return_by_val
    {
        double base_val = 3.25;
        double* ptr = &base_val;
        double** pptr = &ptr;
        **pptr = return_h_double();
        check(**pptr == 7.75, "heldout_double_via_pptr_return_by_val");
        std::printf("CASE 336: heldout_double_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 337: heldout_bool_local_assign
    {
        bool lv = false;
        lv = true;
        check(lv == true, "heldout_bool_local_assign");
        std::printf("CASE 337: heldout_bool_local_assign value=%d\n", (int)(lv));
    }

    // Case 338: heldout_bool_local_compound_assign
    {
        bool lv = false;
        lv = true;
        check(lv == true, "heldout_bool_local_compound_assign");
        std::printf("CASE 338: heldout_bool_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 339: heldout_bool_local_inc_dec
    {
        bool lv = false;
        lv = false;
        check(lv == false, "heldout_bool_local_inc_dec");
        std::printf("CASE 339: heldout_bool_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 340: heldout_bool_local_read_after_write
    {
        bool lv = false;
        lv = true;
        bool read_val = lv;
        check(read_val == true, "heldout_bool_local_read_after_write");
        std::printf("CASE 340: heldout_bool_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 341: heldout_bool_local_pass_by_val
    {
        bool lv = false;
        take_val_h_bool(lv);
        check(lv == false, "heldout_bool_local_pass_by_val");
        std::printf("CASE 341: heldout_bool_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 342: heldout_bool_local_pass_by_ref
    {
        bool lv = false;
        take_ref_h_bool(lv);
        check(lv == false, "heldout_bool_local_pass_by_ref");
        std::printf("CASE 342: heldout_bool_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 343: heldout_bool_local_return_by_val
    {
        bool lv = false;
        lv = return_h_bool();
        check(lv == true, "heldout_bool_local_return_by_val");
        std::printf("CASE 343: heldout_bool_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 344: heldout_bool_global_assign
    {
        h_var_bool = false;
        h_var_bool = true;
        check(h_var_bool == true, "heldout_bool_global_assign");
        std::printf("CASE 344: heldout_bool_global_assign value=%d\n", (int)(h_var_bool));
    }

    // Case 345: heldout_bool_global_compound_assign
    {
        h_var_bool = false;
        h_var_bool = true;
        check(h_var_bool == true, "heldout_bool_global_compound_assign");
        std::printf("CASE 345: heldout_bool_global_compound_assign value=%d\n", (int)(h_var_bool));
    }

    // Case 346: heldout_bool_global_inc_dec
    {
        h_var_bool = false;
        h_var_bool = false;
        check(h_var_bool == false, "heldout_bool_global_inc_dec");
        std::printf("CASE 346: heldout_bool_global_inc_dec value=%d\n", (int)(h_var_bool));
    }

    // Case 347: heldout_bool_global_read_after_write
    {
        h_var_bool = false;
        h_var_bool = true;
        bool read_val = h_var_bool;
        check(read_val == true, "heldout_bool_global_read_after_write");
        std::printf("CASE 347: heldout_bool_global_read_after_write value=%d\n", (int)(h_var_bool));
    }

    // Case 348: heldout_bool_global_pass_by_val
    {
        h_var_bool = false;
        take_val_h_bool(h_var_bool);
        check(h_var_bool == false, "heldout_bool_global_pass_by_val");
        std::printf("CASE 348: heldout_bool_global_pass_by_val value=%d\n", (int)(h_var_bool));
    }

    // Case 349: heldout_bool_global_pass_by_ref
    {
        h_var_bool = false;
        take_ref_h_bool(h_var_bool);
        check(h_var_bool == false, "heldout_bool_global_pass_by_ref");
        std::printf("CASE 349: heldout_bool_global_pass_by_ref value=%d\n", (int)(h_var_bool));
    }

    // Case 350: heldout_bool_global_return_by_val
    {
        h_var_bool = false;
        h_var_bool = return_h_bool();
        check(h_var_bool == true, "heldout_bool_global_return_by_val");
        std::printf("CASE 350: heldout_bool_global_return_by_val value=%d\n", (int)(h_var_bool));
    }

    // Case 351: heldout_bool_static_local_assign
    {
        static bool s_var_bool; s_var_bool = false;
        s_var_bool = true;
        check(s_var_bool == true, "heldout_bool_static_local_assign");
        std::printf("CASE 351: heldout_bool_static_local_assign value=%d\n", (int)(s_var_bool));
    }

    // Case 352: heldout_bool_static_local_compound_assign
    {
        static bool s_var_bool; s_var_bool = false;
        s_var_bool = true;
        check(s_var_bool == true, "heldout_bool_static_local_compound_assign");
        std::printf("CASE 352: heldout_bool_static_local_compound_assign value=%d\n", (int)(s_var_bool));
    }

    // Case 353: heldout_bool_static_local_inc_dec
    {
        static bool s_var_bool; s_var_bool = false;
        s_var_bool = false;
        check(s_var_bool == false, "heldout_bool_static_local_inc_dec");
        std::printf("CASE 353: heldout_bool_static_local_inc_dec value=%d\n", (int)(s_var_bool));
    }

    // Case 354: heldout_bool_static_local_read_after_write
    {
        static bool s_var_bool; s_var_bool = false;
        s_var_bool = true;
        bool read_val = s_var_bool;
        check(read_val == true, "heldout_bool_static_local_read_after_write");
        std::printf("CASE 354: heldout_bool_static_local_read_after_write value=%d\n", (int)(s_var_bool));
    }

    // Case 355: heldout_bool_static_local_pass_by_val
    {
        static bool s_var_bool; s_var_bool = false;
        take_val_h_bool(s_var_bool);
        check(s_var_bool == false, "heldout_bool_static_local_pass_by_val");
        std::printf("CASE 355: heldout_bool_static_local_pass_by_val value=%d\n", (int)(s_var_bool));
    }

    // Case 356: heldout_bool_static_local_pass_by_ref
    {
        static bool s_var_bool; s_var_bool = false;
        take_ref_h_bool(s_var_bool);
        check(s_var_bool == false, "heldout_bool_static_local_pass_by_ref");
        std::printf("CASE 356: heldout_bool_static_local_pass_by_ref value=%d\n", (int)(s_var_bool));
    }

    // Case 357: heldout_bool_static_local_return_by_val
    {
        static bool s_var_bool; s_var_bool = false;
        s_var_bool = return_h_bool();
        check(s_var_bool == true, "heldout_bool_static_local_return_by_val");
        std::printf("CASE 357: heldout_bool_static_local_return_by_val value=%d\n", (int)(s_var_bool));
    }

    // Case 358: heldout_bool_struct_member_assign
    {
        Holder_h_bool h; h.m = false;
        h.m = true;
        check(h.m == true, "heldout_bool_struct_member_assign");
        std::printf("CASE 358: heldout_bool_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 359: heldout_bool_struct_member_compound_assign
    {
        Holder_h_bool h; h.m = false;
        h.m = true;
        check(h.m == true, "heldout_bool_struct_member_compound_assign");
        std::printf("CASE 359: heldout_bool_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 360: heldout_bool_struct_member_inc_dec
    {
        Holder_h_bool h; h.m = false;
        h.m = false;
        check(h.m == false, "heldout_bool_struct_member_inc_dec");
        std::printf("CASE 360: heldout_bool_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 361: heldout_bool_struct_member_read_after_write
    {
        Holder_h_bool h; h.m = false;
        h.m = true;
        bool read_val = h.m;
        check(read_val == true, "heldout_bool_struct_member_read_after_write");
        std::printf("CASE 361: heldout_bool_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 362: heldout_bool_struct_member_pass_by_val
    {
        Holder_h_bool h; h.m = false;
        take_val_h_bool(h.m);
        check(h.m == false, "heldout_bool_struct_member_pass_by_val");
        std::printf("CASE 362: heldout_bool_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 363: heldout_bool_struct_member_pass_by_ref
    {
        Holder_h_bool h; h.m = false;
        take_ref_h_bool(h.m);
        check(h.m == false, "heldout_bool_struct_member_pass_by_ref");
        std::printf("CASE 363: heldout_bool_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 364: heldout_bool_struct_member_return_by_val
    {
        Holder_h_bool h; h.m = false;
        h.m = return_h_bool();
        check(h.m == true, "heldout_bool_struct_member_return_by_val");
        std::printf("CASE 364: heldout_bool_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 365: heldout_bool_array_elem_assign
    {
        bool arr[3] = { false, false, false };
        arr[1] = true;
        check(arr[1] == true, "heldout_bool_array_elem_assign");
        std::printf("CASE 365: heldout_bool_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 366: heldout_bool_array_elem_compound_assign
    {
        bool arr[3] = { false, false, false };
        arr[1] = true;
        check(arr[1] == true, "heldout_bool_array_elem_compound_assign");
        std::printf("CASE 366: heldout_bool_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 367: heldout_bool_array_elem_inc_dec
    {
        bool arr[3] = { false, false, false };
        arr[1] = false;
        check(arr[1] == false, "heldout_bool_array_elem_inc_dec");
        std::printf("CASE 367: heldout_bool_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 368: heldout_bool_array_elem_read_after_write
    {
        bool arr[3] = { false, false, false };
        arr[1] = true;
        bool read_val = arr[1];
        check(read_val == true, "heldout_bool_array_elem_read_after_write");
        std::printf("CASE 368: heldout_bool_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 369: heldout_bool_array_elem_pass_by_val
    {
        bool arr[3] = { false, false, false };
        take_val_h_bool(arr[1]);
        check(arr[1] == false, "heldout_bool_array_elem_pass_by_val");
        std::printf("CASE 369: heldout_bool_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 370: heldout_bool_array_elem_pass_by_ref
    {
        bool arr[3] = { false, false, false };
        take_ref_h_bool(arr[1]);
        check(arr[1] == false, "heldout_bool_array_elem_pass_by_ref");
        std::printf("CASE 370: heldout_bool_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 371: heldout_bool_array_elem_return_by_val
    {
        bool arr[3] = { false, false, false };
        arr[1] = return_h_bool();
        check(arr[1] == true, "heldout_bool_array_elem_return_by_val");
        std::printf("CASE 371: heldout_bool_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 372: heldout_bool_via_ref_assign
    {
        bool base_val = false;
        bool& ref = base_val;
        ref = true;
        check(ref == true, "heldout_bool_via_ref_assign");
        std::printf("CASE 372: heldout_bool_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 373: heldout_bool_via_ref_compound_assign
    {
        bool base_val = false;
        bool& ref = base_val;
        ref = true;
        check(ref == true, "heldout_bool_via_ref_compound_assign");
        std::printf("CASE 373: heldout_bool_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 374: heldout_bool_via_ref_inc_dec
    {
        bool base_val = false;
        bool& ref = base_val;
        ref = false;
        check(ref == false, "heldout_bool_via_ref_inc_dec");
        std::printf("CASE 374: heldout_bool_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 375: heldout_bool_via_ref_read_after_write
    {
        bool base_val = false;
        bool& ref = base_val;
        ref = true;
        bool read_val = ref;
        check(read_val == true, "heldout_bool_via_ref_read_after_write");
        std::printf("CASE 375: heldout_bool_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 376: heldout_bool_via_ref_pass_by_val
    {
        bool base_val = false;
        bool& ref = base_val;
        take_val_h_bool(ref);
        check(ref == false, "heldout_bool_via_ref_pass_by_val");
        std::printf("CASE 376: heldout_bool_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 377: heldout_bool_via_ref_pass_by_ref
    {
        bool base_val = false;
        bool& ref = base_val;
        take_ref_h_bool(ref);
        check(ref == false, "heldout_bool_via_ref_pass_by_ref");
        std::printf("CASE 377: heldout_bool_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 378: heldout_bool_via_ref_return_by_val
    {
        bool base_val = false;
        bool& ref = base_val;
        ref = return_h_bool();
        check(ref == true, "heldout_bool_via_ref_return_by_val");
        std::printf("CASE 378: heldout_bool_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 379: heldout_bool_via_ptr_assign
    {
        bool base_val = false;
        bool* ptr = &base_val;
        *ptr = true;
        check(*ptr == true, "heldout_bool_via_ptr_assign");
        std::printf("CASE 379: heldout_bool_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 380: heldout_bool_via_ptr_compound_assign
    {
        bool base_val = false;
        bool* ptr = &base_val;
        *ptr = true;
        check(*ptr == true, "heldout_bool_via_ptr_compound_assign");
        std::printf("CASE 380: heldout_bool_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 381: heldout_bool_via_ptr_inc_dec
    {
        bool base_val = false;
        bool* ptr = &base_val;
        *ptr = false;
        check(*ptr == false, "heldout_bool_via_ptr_inc_dec");
        std::printf("CASE 381: heldout_bool_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 382: heldout_bool_via_ptr_read_after_write
    {
        bool base_val = false;
        bool* ptr = &base_val;
        *ptr = true;
        bool read_val = *ptr;
        check(read_val == true, "heldout_bool_via_ptr_read_after_write");
        std::printf("CASE 382: heldout_bool_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 383: heldout_bool_via_ptr_pass_by_val
    {
        bool base_val = false;
        bool* ptr = &base_val;
        take_val_h_bool(*ptr);
        check(*ptr == false, "heldout_bool_via_ptr_pass_by_val");
        std::printf("CASE 383: heldout_bool_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 384: heldout_bool_via_ptr_pass_by_ref
    {
        bool base_val = false;
        bool* ptr = &base_val;
        take_ref_h_bool(*ptr);
        check(*ptr == false, "heldout_bool_via_ptr_pass_by_ref");
        std::printf("CASE 384: heldout_bool_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 385: heldout_bool_via_ptr_return_by_val
    {
        bool base_val = false;
        bool* ptr = &base_val;
        *ptr = return_h_bool();
        check(*ptr == true, "heldout_bool_via_ptr_return_by_val");
        std::printf("CASE 385: heldout_bool_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 386: heldout_bool_via_pptr_assign
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = true;
        check(**pptr == true, "heldout_bool_via_pptr_assign");
        std::printf("CASE 386: heldout_bool_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 387: heldout_bool_via_pptr_compound_assign
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = true;
        check(**pptr == true, "heldout_bool_via_pptr_compound_assign");
        std::printf("CASE 387: heldout_bool_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 388: heldout_bool_via_pptr_inc_dec
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = false;
        check(**pptr == false, "heldout_bool_via_pptr_inc_dec");
        std::printf("CASE 388: heldout_bool_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 389: heldout_bool_via_pptr_read_after_write
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = true;
        bool read_val = **pptr;
        check(read_val == true, "heldout_bool_via_pptr_read_after_write");
        std::printf("CASE 389: heldout_bool_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 390: heldout_bool_via_pptr_pass_by_val
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        take_val_h_bool(**pptr);
        check(**pptr == false, "heldout_bool_via_pptr_pass_by_val");
        std::printf("CASE 390: heldout_bool_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 391: heldout_bool_via_pptr_pass_by_ref
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        take_ref_h_bool(**pptr);
        check(**pptr == false, "heldout_bool_via_pptr_pass_by_ref");
        std::printf("CASE 391: heldout_bool_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 392: heldout_bool_via_pptr_return_by_val
    {
        bool base_val = false;
        bool* ptr = &base_val;
        bool** pptr = &ptr;
        **pptr = return_h_bool();
        check(**pptr == true, "heldout_bool_via_pptr_return_by_val");
        std::printf("CASE 392: heldout_bool_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    // Case 393: heldout_char_ptr_local_assign
    {
        char* lv = h_char_buf + 1;
        lv = h_char_buf + 3;
        check(lv == h_char_buf + 3, "heldout_char_ptr_local_assign");
        std::printf("CASE 393: heldout_char_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 394: heldout_char_ptr_local_compound_assign
    {
        char* lv = h_char_buf + 1;
        lv += 1;
        check(lv == (h_char_buf + 1) + 1, "heldout_char_ptr_local_compound_assign");
        std::printf("CASE 394: heldout_char_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 395: heldout_char_ptr_local_inc_dec
    {
        char* lv = h_char_buf + 1;
        ++lv;
        --lv;
        check(lv == h_char_buf + 1, "heldout_char_ptr_local_inc_dec");
        std::printf("CASE 395: heldout_char_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 396: heldout_char_ptr_local_read_after_write
    {
        char* lv = h_char_buf + 1;
        lv = h_char_buf + 3;
        char* read_val = lv;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_local_read_after_write");
        std::printf("CASE 396: heldout_char_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 397: heldout_char_ptr_local_pass_by_val
    {
        char* lv = h_char_buf + 1;
        take_val_h_char_ptr(lv);
        check(lv == h_char_buf + 1, "heldout_char_ptr_local_pass_by_val");
        std::printf("CASE 397: heldout_char_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 398: heldout_char_ptr_local_pass_by_ref
    {
        char* lv = h_char_buf + 1;
        take_ref_h_char_ptr(lv);
        check(lv == h_char_buf + 1, "heldout_char_ptr_local_pass_by_ref");
        std::printf("CASE 398: heldout_char_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 399: heldout_char_ptr_local_return_by_val
    {
        char* lv = h_char_buf + 1;
        lv = return_h_char_ptr();
        check(lv == h_char_buf + 3, "heldout_char_ptr_local_return_by_val");
        std::printf("CASE 399: heldout_char_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 400: heldout_char_ptr_global_assign
    {
        h_var_char_ptr = h_char_buf + 1;
        h_var_char_ptr = h_char_buf + 3;
        check(h_var_char_ptr == h_char_buf + 3, "heldout_char_ptr_global_assign");
        std::printf("CASE 400: heldout_char_ptr_global_assign value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 401: heldout_char_ptr_global_compound_assign
    {
        h_var_char_ptr = h_char_buf + 1;
        h_var_char_ptr += 1;
        check(h_var_char_ptr == (h_char_buf + 1) + 1, "heldout_char_ptr_global_compound_assign");
        std::printf("CASE 401: heldout_char_ptr_global_compound_assign value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 402: heldout_char_ptr_global_inc_dec
    {
        h_var_char_ptr = h_char_buf + 1;
        ++h_var_char_ptr;
        --h_var_char_ptr;
        check(h_var_char_ptr == h_char_buf + 1, "heldout_char_ptr_global_inc_dec");
        std::printf("CASE 402: heldout_char_ptr_global_inc_dec value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 403: heldout_char_ptr_global_read_after_write
    {
        h_var_char_ptr = h_char_buf + 1;
        h_var_char_ptr = h_char_buf + 3;
        char* read_val = h_var_char_ptr;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_global_read_after_write");
        std::printf("CASE 403: heldout_char_ptr_global_read_after_write value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 404: heldout_char_ptr_global_pass_by_val
    {
        h_var_char_ptr = h_char_buf + 1;
        take_val_h_char_ptr(h_var_char_ptr);
        check(h_var_char_ptr == h_char_buf + 1, "heldout_char_ptr_global_pass_by_val");
        std::printf("CASE 404: heldout_char_ptr_global_pass_by_val value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 405: heldout_char_ptr_global_pass_by_ref
    {
        h_var_char_ptr = h_char_buf + 1;
        take_ref_h_char_ptr(h_var_char_ptr);
        check(h_var_char_ptr == h_char_buf + 1, "heldout_char_ptr_global_pass_by_ref");
        std::printf("CASE 405: heldout_char_ptr_global_pass_by_ref value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 406: heldout_char_ptr_global_return_by_val
    {
        h_var_char_ptr = h_char_buf + 1;
        h_var_char_ptr = return_h_char_ptr();
        check(h_var_char_ptr == h_char_buf + 3, "heldout_char_ptr_global_return_by_val");
        std::printf("CASE 406: heldout_char_ptr_global_return_by_val value=%d\n", (h_var_char_ptr != nullptr));
    }

    // Case 407: heldout_char_ptr_static_local_assign
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        s_var_char_ptr = h_char_buf + 3;
        check(s_var_char_ptr == h_char_buf + 3, "heldout_char_ptr_static_local_assign");
        std::printf("CASE 407: heldout_char_ptr_static_local_assign value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 408: heldout_char_ptr_static_local_compound_assign
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        s_var_char_ptr += 1;
        check(s_var_char_ptr == (h_char_buf + 1) + 1, "heldout_char_ptr_static_local_compound_assign");
        std::printf("CASE 408: heldout_char_ptr_static_local_compound_assign value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 409: heldout_char_ptr_static_local_inc_dec
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        ++s_var_char_ptr;
        --s_var_char_ptr;
        check(s_var_char_ptr == h_char_buf + 1, "heldout_char_ptr_static_local_inc_dec");
        std::printf("CASE 409: heldout_char_ptr_static_local_inc_dec value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 410: heldout_char_ptr_static_local_read_after_write
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        s_var_char_ptr = h_char_buf + 3;
        char* read_val = s_var_char_ptr;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_static_local_read_after_write");
        std::printf("CASE 410: heldout_char_ptr_static_local_read_after_write value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 411: heldout_char_ptr_static_local_pass_by_val
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        take_val_h_char_ptr(s_var_char_ptr);
        check(s_var_char_ptr == h_char_buf + 1, "heldout_char_ptr_static_local_pass_by_val");
        std::printf("CASE 411: heldout_char_ptr_static_local_pass_by_val value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 412: heldout_char_ptr_static_local_pass_by_ref
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        take_ref_h_char_ptr(s_var_char_ptr);
        check(s_var_char_ptr == h_char_buf + 1, "heldout_char_ptr_static_local_pass_by_ref");
        std::printf("CASE 412: heldout_char_ptr_static_local_pass_by_ref value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 413: heldout_char_ptr_static_local_return_by_val
    {
        static char* s_var_char_ptr; s_var_char_ptr = h_char_buf + 1;
        s_var_char_ptr = return_h_char_ptr();
        check(s_var_char_ptr == h_char_buf + 3, "heldout_char_ptr_static_local_return_by_val");
        std::printf("CASE 413: heldout_char_ptr_static_local_return_by_val value=%d\n", (s_var_char_ptr != nullptr));
    }

    // Case 414: heldout_char_ptr_struct_member_assign
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        h.m = h_char_buf + 3;
        check(h.m == h_char_buf + 3, "heldout_char_ptr_struct_member_assign");
        std::printf("CASE 414: heldout_char_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 415: heldout_char_ptr_struct_member_compound_assign
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        h.m += 1;
        check(h.m == (h_char_buf + 1) + 1, "heldout_char_ptr_struct_member_compound_assign");
        std::printf("CASE 415: heldout_char_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 416: heldout_char_ptr_struct_member_inc_dec
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        ++h.m;
        --h.m;
        check(h.m == h_char_buf + 1, "heldout_char_ptr_struct_member_inc_dec");
        std::printf("CASE 416: heldout_char_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 417: heldout_char_ptr_struct_member_read_after_write
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        h.m = h_char_buf + 3;
        char* read_val = h.m;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_struct_member_read_after_write");
        std::printf("CASE 417: heldout_char_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 418: heldout_char_ptr_struct_member_pass_by_val
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        take_val_h_char_ptr(h.m);
        check(h.m == h_char_buf + 1, "heldout_char_ptr_struct_member_pass_by_val");
        std::printf("CASE 418: heldout_char_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 419: heldout_char_ptr_struct_member_pass_by_ref
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        take_ref_h_char_ptr(h.m);
        check(h.m == h_char_buf + 1, "heldout_char_ptr_struct_member_pass_by_ref");
        std::printf("CASE 419: heldout_char_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 420: heldout_char_ptr_struct_member_return_by_val
    {
        Holder_h_char_ptr h; h.m = h_char_buf + 1;
        h.m = return_h_char_ptr();
        check(h.m == h_char_buf + 3, "heldout_char_ptr_struct_member_return_by_val");
        std::printf("CASE 420: heldout_char_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 421: heldout_char_ptr_array_elem_assign
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        arr[1] = h_char_buf + 3;
        check(arr[1] == h_char_buf + 3, "heldout_char_ptr_array_elem_assign");
        std::printf("CASE 421: heldout_char_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 422: heldout_char_ptr_array_elem_compound_assign
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        arr[1] += 1;
        check(arr[1] == (h_char_buf + 1) + 1, "heldout_char_ptr_array_elem_compound_assign");
        std::printf("CASE 422: heldout_char_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 423: heldout_char_ptr_array_elem_inc_dec
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        ++arr[1];
        --arr[1];
        check(arr[1] == h_char_buf + 1, "heldout_char_ptr_array_elem_inc_dec");
        std::printf("CASE 423: heldout_char_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 424: heldout_char_ptr_array_elem_read_after_write
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        arr[1] = h_char_buf + 3;
        char* read_val = arr[1];
        check(read_val == h_char_buf + 3, "heldout_char_ptr_array_elem_read_after_write");
        std::printf("CASE 424: heldout_char_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 425: heldout_char_ptr_array_elem_pass_by_val
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        take_val_h_char_ptr(arr[1]);
        check(arr[1] == h_char_buf + 1, "heldout_char_ptr_array_elem_pass_by_val");
        std::printf("CASE 425: heldout_char_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 426: heldout_char_ptr_array_elem_pass_by_ref
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        take_ref_h_char_ptr(arr[1]);
        check(arr[1] == h_char_buf + 1, "heldout_char_ptr_array_elem_pass_by_ref");
        std::printf("CASE 426: heldout_char_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 427: heldout_char_ptr_array_elem_return_by_val
    {
        char* arr[3] = { h_char_buf + 1, h_char_buf + 1, h_char_buf + 1 };
        arr[1] = return_h_char_ptr();
        check(arr[1] == h_char_buf + 3, "heldout_char_ptr_array_elem_return_by_val");
        std::printf("CASE 427: heldout_char_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 428: heldout_char_ptr_via_ref_assign
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        ref = h_char_buf + 3;
        check(ref == h_char_buf + 3, "heldout_char_ptr_via_ref_assign");
        std::printf("CASE 428: heldout_char_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 429: heldout_char_ptr_via_ref_compound_assign
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        ref += 1;
        check(ref == (h_char_buf + 1) + 1, "heldout_char_ptr_via_ref_compound_assign");
        std::printf("CASE 429: heldout_char_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 430: heldout_char_ptr_via_ref_inc_dec
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        ++ref;
        --ref;
        check(ref == h_char_buf + 1, "heldout_char_ptr_via_ref_inc_dec");
        std::printf("CASE 430: heldout_char_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 431: heldout_char_ptr_via_ref_read_after_write
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        ref = h_char_buf + 3;
        char* read_val = ref;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_via_ref_read_after_write");
        std::printf("CASE 431: heldout_char_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 432: heldout_char_ptr_via_ref_pass_by_val
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        take_val_h_char_ptr(ref);
        check(ref == h_char_buf + 1, "heldout_char_ptr_via_ref_pass_by_val");
        std::printf("CASE 432: heldout_char_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 433: heldout_char_ptr_via_ref_pass_by_ref
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        take_ref_h_char_ptr(ref);
        check(ref == h_char_buf + 1, "heldout_char_ptr_via_ref_pass_by_ref");
        std::printf("CASE 433: heldout_char_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 434: heldout_char_ptr_via_ref_return_by_val
    {
        char* base_val = h_char_buf + 1;
        char*& ref = base_val;
        ref = return_h_char_ptr();
        check(ref == h_char_buf + 3, "heldout_char_ptr_via_ref_return_by_val");
        std::printf("CASE 434: heldout_char_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 435: heldout_char_ptr_via_ptr_assign
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        *ptr = h_char_buf + 3;
        check(*ptr == h_char_buf + 3, "heldout_char_ptr_via_ptr_assign");
        std::printf("CASE 435: heldout_char_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 436: heldout_char_ptr_via_ptr_compound_assign
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (h_char_buf + 1) + 1, "heldout_char_ptr_via_ptr_compound_assign");
        std::printf("CASE 436: heldout_char_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 437: heldout_char_ptr_via_ptr_inc_dec
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == h_char_buf + 1, "heldout_char_ptr_via_ptr_inc_dec");
        std::printf("CASE 437: heldout_char_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 438: heldout_char_ptr_via_ptr_read_after_write
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        *ptr = h_char_buf + 3;
        char* read_val = *ptr;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_via_ptr_read_after_write");
        std::printf("CASE 438: heldout_char_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 439: heldout_char_ptr_via_ptr_pass_by_val
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        take_val_h_char_ptr(*ptr);
        check(*ptr == h_char_buf + 1, "heldout_char_ptr_via_ptr_pass_by_val");
        std::printf("CASE 439: heldout_char_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 440: heldout_char_ptr_via_ptr_pass_by_ref
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        take_ref_h_char_ptr(*ptr);
        check(*ptr == h_char_buf + 1, "heldout_char_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 440: heldout_char_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 441: heldout_char_ptr_via_ptr_return_by_val
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        *ptr = return_h_char_ptr();
        check(*ptr == h_char_buf + 3, "heldout_char_ptr_via_ptr_return_by_val");
        std::printf("CASE 441: heldout_char_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 442: heldout_char_ptr_via_pptr_assign
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr = h_char_buf + 3;
        check(**pptr == h_char_buf + 3, "heldout_char_ptr_via_pptr_assign");
        std::printf("CASE 442: heldout_char_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 443: heldout_char_ptr_via_pptr_compound_assign
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (h_char_buf + 1) + 1, "heldout_char_ptr_via_pptr_compound_assign");
        std::printf("CASE 443: heldout_char_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 444: heldout_char_ptr_via_pptr_inc_dec
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == h_char_buf + 1, "heldout_char_ptr_via_pptr_inc_dec");
        std::printf("CASE 444: heldout_char_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 445: heldout_char_ptr_via_pptr_read_after_write
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr = h_char_buf + 3;
        char* read_val = **pptr;
        check(read_val == h_char_buf + 3, "heldout_char_ptr_via_pptr_read_after_write");
        std::printf("CASE 445: heldout_char_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 446: heldout_char_ptr_via_pptr_pass_by_val
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        take_val_h_char_ptr(**pptr);
        check(**pptr == h_char_buf + 1, "heldout_char_ptr_via_pptr_pass_by_val");
        std::printf("CASE 446: heldout_char_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 447: heldout_char_ptr_via_pptr_pass_by_ref
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        take_ref_h_char_ptr(**pptr);
        check(**pptr == h_char_buf + 1, "heldout_char_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 447: heldout_char_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 448: heldout_char_ptr_via_pptr_return_by_val
    {
        char* base_val = h_char_buf + 1;
        char** ptr = &base_val;
        char*** pptr = &ptr;
        **pptr = return_h_char_ptr();
        check(**pptr == h_char_buf + 3, "heldout_char_ptr_via_pptr_return_by_val");
        std::printf("CASE 448: heldout_char_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 449: heldout_int_ptr_local_assign
    {
        int* lv = h_int_buf + 1;
        lv = h_int_buf + 3;
        check(lv == h_int_buf + 3, "heldout_int_ptr_local_assign");
        std::printf("CASE 449: heldout_int_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 450: heldout_int_ptr_local_compound_assign
    {
        int* lv = h_int_buf + 1;
        lv += 1;
        check(lv == (h_int_buf + 1) + 1, "heldout_int_ptr_local_compound_assign");
        std::printf("CASE 450: heldout_int_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 451: heldout_int_ptr_local_inc_dec
    {
        int* lv = h_int_buf + 1;
        ++lv;
        --lv;
        check(lv == h_int_buf + 1, "heldout_int_ptr_local_inc_dec");
        std::printf("CASE 451: heldout_int_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 452: heldout_int_ptr_local_read_after_write
    {
        int* lv = h_int_buf + 1;
        lv = h_int_buf + 3;
        int* read_val = lv;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_local_read_after_write");
        std::printf("CASE 452: heldout_int_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 453: heldout_int_ptr_local_pass_by_val
    {
        int* lv = h_int_buf + 1;
        take_val_h_int_ptr(lv);
        check(lv == h_int_buf + 1, "heldout_int_ptr_local_pass_by_val");
        std::printf("CASE 453: heldout_int_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 454: heldout_int_ptr_local_pass_by_ref
    {
        int* lv = h_int_buf + 1;
        take_ref_h_int_ptr(lv);
        check(lv == h_int_buf + 1, "heldout_int_ptr_local_pass_by_ref");
        std::printf("CASE 454: heldout_int_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 455: heldout_int_ptr_local_return_by_val
    {
        int* lv = h_int_buf + 1;
        lv = return_h_int_ptr();
        check(lv == h_int_buf + 3, "heldout_int_ptr_local_return_by_val");
        std::printf("CASE 455: heldout_int_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 456: heldout_int_ptr_global_assign
    {
        h_var_int_ptr = h_int_buf + 1;
        h_var_int_ptr = h_int_buf + 3;
        check(h_var_int_ptr == h_int_buf + 3, "heldout_int_ptr_global_assign");
        std::printf("CASE 456: heldout_int_ptr_global_assign value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 457: heldout_int_ptr_global_compound_assign
    {
        h_var_int_ptr = h_int_buf + 1;
        h_var_int_ptr += 1;
        check(h_var_int_ptr == (h_int_buf + 1) + 1, "heldout_int_ptr_global_compound_assign");
        std::printf("CASE 457: heldout_int_ptr_global_compound_assign value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 458: heldout_int_ptr_global_inc_dec
    {
        h_var_int_ptr = h_int_buf + 1;
        ++h_var_int_ptr;
        --h_var_int_ptr;
        check(h_var_int_ptr == h_int_buf + 1, "heldout_int_ptr_global_inc_dec");
        std::printf("CASE 458: heldout_int_ptr_global_inc_dec value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 459: heldout_int_ptr_global_read_after_write
    {
        h_var_int_ptr = h_int_buf + 1;
        h_var_int_ptr = h_int_buf + 3;
        int* read_val = h_var_int_ptr;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_global_read_after_write");
        std::printf("CASE 459: heldout_int_ptr_global_read_after_write value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 460: heldout_int_ptr_global_pass_by_val
    {
        h_var_int_ptr = h_int_buf + 1;
        take_val_h_int_ptr(h_var_int_ptr);
        check(h_var_int_ptr == h_int_buf + 1, "heldout_int_ptr_global_pass_by_val");
        std::printf("CASE 460: heldout_int_ptr_global_pass_by_val value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 461: heldout_int_ptr_global_pass_by_ref
    {
        h_var_int_ptr = h_int_buf + 1;
        take_ref_h_int_ptr(h_var_int_ptr);
        check(h_var_int_ptr == h_int_buf + 1, "heldout_int_ptr_global_pass_by_ref");
        std::printf("CASE 461: heldout_int_ptr_global_pass_by_ref value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 462: heldout_int_ptr_global_return_by_val
    {
        h_var_int_ptr = h_int_buf + 1;
        h_var_int_ptr = return_h_int_ptr();
        check(h_var_int_ptr == h_int_buf + 3, "heldout_int_ptr_global_return_by_val");
        std::printf("CASE 462: heldout_int_ptr_global_return_by_val value=%d\n", (h_var_int_ptr != nullptr));
    }

    // Case 463: heldout_int_ptr_static_local_assign
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        s_var_int_ptr = h_int_buf + 3;
        check(s_var_int_ptr == h_int_buf + 3, "heldout_int_ptr_static_local_assign");
        std::printf("CASE 463: heldout_int_ptr_static_local_assign value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 464: heldout_int_ptr_static_local_compound_assign
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        s_var_int_ptr += 1;
        check(s_var_int_ptr == (h_int_buf + 1) + 1, "heldout_int_ptr_static_local_compound_assign");
        std::printf("CASE 464: heldout_int_ptr_static_local_compound_assign value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 465: heldout_int_ptr_static_local_inc_dec
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        ++s_var_int_ptr;
        --s_var_int_ptr;
        check(s_var_int_ptr == h_int_buf + 1, "heldout_int_ptr_static_local_inc_dec");
        std::printf("CASE 465: heldout_int_ptr_static_local_inc_dec value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 466: heldout_int_ptr_static_local_read_after_write
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        s_var_int_ptr = h_int_buf + 3;
        int* read_val = s_var_int_ptr;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_static_local_read_after_write");
        std::printf("CASE 466: heldout_int_ptr_static_local_read_after_write value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 467: heldout_int_ptr_static_local_pass_by_val
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        take_val_h_int_ptr(s_var_int_ptr);
        check(s_var_int_ptr == h_int_buf + 1, "heldout_int_ptr_static_local_pass_by_val");
        std::printf("CASE 467: heldout_int_ptr_static_local_pass_by_val value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 468: heldout_int_ptr_static_local_pass_by_ref
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        take_ref_h_int_ptr(s_var_int_ptr);
        check(s_var_int_ptr == h_int_buf + 1, "heldout_int_ptr_static_local_pass_by_ref");
        std::printf("CASE 468: heldout_int_ptr_static_local_pass_by_ref value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 469: heldout_int_ptr_static_local_return_by_val
    {
        static int* s_var_int_ptr; s_var_int_ptr = h_int_buf + 1;
        s_var_int_ptr = return_h_int_ptr();
        check(s_var_int_ptr == h_int_buf + 3, "heldout_int_ptr_static_local_return_by_val");
        std::printf("CASE 469: heldout_int_ptr_static_local_return_by_val value=%d\n", (s_var_int_ptr != nullptr));
    }

    // Case 470: heldout_int_ptr_struct_member_assign
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        h.m = h_int_buf + 3;
        check(h.m == h_int_buf + 3, "heldout_int_ptr_struct_member_assign");
        std::printf("CASE 470: heldout_int_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 471: heldout_int_ptr_struct_member_compound_assign
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        h.m += 1;
        check(h.m == (h_int_buf + 1) + 1, "heldout_int_ptr_struct_member_compound_assign");
        std::printf("CASE 471: heldout_int_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 472: heldout_int_ptr_struct_member_inc_dec
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        ++h.m;
        --h.m;
        check(h.m == h_int_buf + 1, "heldout_int_ptr_struct_member_inc_dec");
        std::printf("CASE 472: heldout_int_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 473: heldout_int_ptr_struct_member_read_after_write
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        h.m = h_int_buf + 3;
        int* read_val = h.m;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_struct_member_read_after_write");
        std::printf("CASE 473: heldout_int_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 474: heldout_int_ptr_struct_member_pass_by_val
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        take_val_h_int_ptr(h.m);
        check(h.m == h_int_buf + 1, "heldout_int_ptr_struct_member_pass_by_val");
        std::printf("CASE 474: heldout_int_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 475: heldout_int_ptr_struct_member_pass_by_ref
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        take_ref_h_int_ptr(h.m);
        check(h.m == h_int_buf + 1, "heldout_int_ptr_struct_member_pass_by_ref");
        std::printf("CASE 475: heldout_int_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 476: heldout_int_ptr_struct_member_return_by_val
    {
        Holder_h_int_ptr h; h.m = h_int_buf + 1;
        h.m = return_h_int_ptr();
        check(h.m == h_int_buf + 3, "heldout_int_ptr_struct_member_return_by_val");
        std::printf("CASE 476: heldout_int_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 477: heldout_int_ptr_array_elem_assign
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        arr[1] = h_int_buf + 3;
        check(arr[1] == h_int_buf + 3, "heldout_int_ptr_array_elem_assign");
        std::printf("CASE 477: heldout_int_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 478: heldout_int_ptr_array_elem_compound_assign
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        arr[1] += 1;
        check(arr[1] == (h_int_buf + 1) + 1, "heldout_int_ptr_array_elem_compound_assign");
        std::printf("CASE 478: heldout_int_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 479: heldout_int_ptr_array_elem_inc_dec
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        ++arr[1];
        --arr[1];
        check(arr[1] == h_int_buf + 1, "heldout_int_ptr_array_elem_inc_dec");
        std::printf("CASE 479: heldout_int_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 480: heldout_int_ptr_array_elem_read_after_write
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        arr[1] = h_int_buf + 3;
        int* read_val = arr[1];
        check(read_val == h_int_buf + 3, "heldout_int_ptr_array_elem_read_after_write");
        std::printf("CASE 480: heldout_int_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 481: heldout_int_ptr_array_elem_pass_by_val
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        take_val_h_int_ptr(arr[1]);
        check(arr[1] == h_int_buf + 1, "heldout_int_ptr_array_elem_pass_by_val");
        std::printf("CASE 481: heldout_int_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 482: heldout_int_ptr_array_elem_pass_by_ref
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        take_ref_h_int_ptr(arr[1]);
        check(arr[1] == h_int_buf + 1, "heldout_int_ptr_array_elem_pass_by_ref");
        std::printf("CASE 482: heldout_int_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 483: heldout_int_ptr_array_elem_return_by_val
    {
        int* arr[3] = { h_int_buf + 1, h_int_buf + 1, h_int_buf + 1 };
        arr[1] = return_h_int_ptr();
        check(arr[1] == h_int_buf + 3, "heldout_int_ptr_array_elem_return_by_val");
        std::printf("CASE 483: heldout_int_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 484: heldout_int_ptr_via_ref_assign
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        ref = h_int_buf + 3;
        check(ref == h_int_buf + 3, "heldout_int_ptr_via_ref_assign");
        std::printf("CASE 484: heldout_int_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 485: heldout_int_ptr_via_ref_compound_assign
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        ref += 1;
        check(ref == (h_int_buf + 1) + 1, "heldout_int_ptr_via_ref_compound_assign");
        std::printf("CASE 485: heldout_int_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 486: heldout_int_ptr_via_ref_inc_dec
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        ++ref;
        --ref;
        check(ref == h_int_buf + 1, "heldout_int_ptr_via_ref_inc_dec");
        std::printf("CASE 486: heldout_int_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 487: heldout_int_ptr_via_ref_read_after_write
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        ref = h_int_buf + 3;
        int* read_val = ref;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_via_ref_read_after_write");
        std::printf("CASE 487: heldout_int_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 488: heldout_int_ptr_via_ref_pass_by_val
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        take_val_h_int_ptr(ref);
        check(ref == h_int_buf + 1, "heldout_int_ptr_via_ref_pass_by_val");
        std::printf("CASE 488: heldout_int_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 489: heldout_int_ptr_via_ref_pass_by_ref
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        take_ref_h_int_ptr(ref);
        check(ref == h_int_buf + 1, "heldout_int_ptr_via_ref_pass_by_ref");
        std::printf("CASE 489: heldout_int_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 490: heldout_int_ptr_via_ref_return_by_val
    {
        int* base_val = h_int_buf + 1;
        int*& ref = base_val;
        ref = return_h_int_ptr();
        check(ref == h_int_buf + 3, "heldout_int_ptr_via_ref_return_by_val");
        std::printf("CASE 490: heldout_int_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 491: heldout_int_ptr_via_ptr_assign
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        *ptr = h_int_buf + 3;
        check(*ptr == h_int_buf + 3, "heldout_int_ptr_via_ptr_assign");
        std::printf("CASE 491: heldout_int_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 492: heldout_int_ptr_via_ptr_compound_assign
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (h_int_buf + 1) + 1, "heldout_int_ptr_via_ptr_compound_assign");
        std::printf("CASE 492: heldout_int_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 493: heldout_int_ptr_via_ptr_inc_dec
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == h_int_buf + 1, "heldout_int_ptr_via_ptr_inc_dec");
        std::printf("CASE 493: heldout_int_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 494: heldout_int_ptr_via_ptr_read_after_write
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        *ptr = h_int_buf + 3;
        int* read_val = *ptr;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_via_ptr_read_after_write");
        std::printf("CASE 494: heldout_int_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 495: heldout_int_ptr_via_ptr_pass_by_val
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        take_val_h_int_ptr(*ptr);
        check(*ptr == h_int_buf + 1, "heldout_int_ptr_via_ptr_pass_by_val");
        std::printf("CASE 495: heldout_int_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 496: heldout_int_ptr_via_ptr_pass_by_ref
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        take_ref_h_int_ptr(*ptr);
        check(*ptr == h_int_buf + 1, "heldout_int_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 496: heldout_int_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 497: heldout_int_ptr_via_ptr_return_by_val
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        *ptr = return_h_int_ptr();
        check(*ptr == h_int_buf + 3, "heldout_int_ptr_via_ptr_return_by_val");
        std::printf("CASE 497: heldout_int_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 498: heldout_int_ptr_via_pptr_assign
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr = h_int_buf + 3;
        check(**pptr == h_int_buf + 3, "heldout_int_ptr_via_pptr_assign");
        std::printf("CASE 498: heldout_int_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 499: heldout_int_ptr_via_pptr_compound_assign
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (h_int_buf + 1) + 1, "heldout_int_ptr_via_pptr_compound_assign");
        std::printf("CASE 499: heldout_int_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 500: heldout_int_ptr_via_pptr_inc_dec
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == h_int_buf + 1, "heldout_int_ptr_via_pptr_inc_dec");
        std::printf("CASE 500: heldout_int_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 501: heldout_int_ptr_via_pptr_read_after_write
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr = h_int_buf + 3;
        int* read_val = **pptr;
        check(read_val == h_int_buf + 3, "heldout_int_ptr_via_pptr_read_after_write");
        std::printf("CASE 501: heldout_int_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 502: heldout_int_ptr_via_pptr_pass_by_val
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        take_val_h_int_ptr(**pptr);
        check(**pptr == h_int_buf + 1, "heldout_int_ptr_via_pptr_pass_by_val");
        std::printf("CASE 502: heldout_int_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 503: heldout_int_ptr_via_pptr_pass_by_ref
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        take_ref_h_int_ptr(**pptr);
        check(**pptr == h_int_buf + 1, "heldout_int_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 503: heldout_int_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 504: heldout_int_ptr_via_pptr_return_by_val
    {
        int* base_val = h_int_buf + 1;
        int** ptr = &base_val;
        int*** pptr = &ptr;
        **pptr = return_h_int_ptr();
        check(**pptr == h_int_buf + 3, "heldout_int_ptr_via_pptr_return_by_val");
        std::printf("CASE 504: heldout_int_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 505: heldout_double_ptr_local_assign
    {
        double* lv = h_double_buf + 1;
        lv = h_double_buf + 3;
        check(lv == h_double_buf + 3, "heldout_double_ptr_local_assign");
        std::printf("CASE 505: heldout_double_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 506: heldout_double_ptr_local_compound_assign
    {
        double* lv = h_double_buf + 1;
        lv += 1;
        check(lv == (h_double_buf + 1) + 1, "heldout_double_ptr_local_compound_assign");
        std::printf("CASE 506: heldout_double_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 507: heldout_double_ptr_local_inc_dec
    {
        double* lv = h_double_buf + 1;
        ++lv;
        --lv;
        check(lv == h_double_buf + 1, "heldout_double_ptr_local_inc_dec");
        std::printf("CASE 507: heldout_double_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 508: heldout_double_ptr_local_read_after_write
    {
        double* lv = h_double_buf + 1;
        lv = h_double_buf + 3;
        double* read_val = lv;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_local_read_after_write");
        std::printf("CASE 508: heldout_double_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 509: heldout_double_ptr_local_pass_by_val
    {
        double* lv = h_double_buf + 1;
        take_val_h_double_ptr(lv);
        check(lv == h_double_buf + 1, "heldout_double_ptr_local_pass_by_val");
        std::printf("CASE 509: heldout_double_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 510: heldout_double_ptr_local_pass_by_ref
    {
        double* lv = h_double_buf + 1;
        take_ref_h_double_ptr(lv);
        check(lv == h_double_buf + 1, "heldout_double_ptr_local_pass_by_ref");
        std::printf("CASE 510: heldout_double_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 511: heldout_double_ptr_local_return_by_val
    {
        double* lv = h_double_buf + 1;
        lv = return_h_double_ptr();
        check(lv == h_double_buf + 3, "heldout_double_ptr_local_return_by_val");
        std::printf("CASE 511: heldout_double_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 512: heldout_double_ptr_global_assign
    {
        h_var_double_ptr = h_double_buf + 1;
        h_var_double_ptr = h_double_buf + 3;
        check(h_var_double_ptr == h_double_buf + 3, "heldout_double_ptr_global_assign");
        std::printf("CASE 512: heldout_double_ptr_global_assign value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 513: heldout_double_ptr_global_compound_assign
    {
        h_var_double_ptr = h_double_buf + 1;
        h_var_double_ptr += 1;
        check(h_var_double_ptr == (h_double_buf + 1) + 1, "heldout_double_ptr_global_compound_assign");
        std::printf("CASE 513: heldout_double_ptr_global_compound_assign value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 514: heldout_double_ptr_global_inc_dec
    {
        h_var_double_ptr = h_double_buf + 1;
        ++h_var_double_ptr;
        --h_var_double_ptr;
        check(h_var_double_ptr == h_double_buf + 1, "heldout_double_ptr_global_inc_dec");
        std::printf("CASE 514: heldout_double_ptr_global_inc_dec value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 515: heldout_double_ptr_global_read_after_write
    {
        h_var_double_ptr = h_double_buf + 1;
        h_var_double_ptr = h_double_buf + 3;
        double* read_val = h_var_double_ptr;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_global_read_after_write");
        std::printf("CASE 515: heldout_double_ptr_global_read_after_write value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 516: heldout_double_ptr_global_pass_by_val
    {
        h_var_double_ptr = h_double_buf + 1;
        take_val_h_double_ptr(h_var_double_ptr);
        check(h_var_double_ptr == h_double_buf + 1, "heldout_double_ptr_global_pass_by_val");
        std::printf("CASE 516: heldout_double_ptr_global_pass_by_val value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 517: heldout_double_ptr_global_pass_by_ref
    {
        h_var_double_ptr = h_double_buf + 1;
        take_ref_h_double_ptr(h_var_double_ptr);
        check(h_var_double_ptr == h_double_buf + 1, "heldout_double_ptr_global_pass_by_ref");
        std::printf("CASE 517: heldout_double_ptr_global_pass_by_ref value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 518: heldout_double_ptr_global_return_by_val
    {
        h_var_double_ptr = h_double_buf + 1;
        h_var_double_ptr = return_h_double_ptr();
        check(h_var_double_ptr == h_double_buf + 3, "heldout_double_ptr_global_return_by_val");
        std::printf("CASE 518: heldout_double_ptr_global_return_by_val value=%d\n", (h_var_double_ptr != nullptr));
    }

    // Case 519: heldout_double_ptr_static_local_assign
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        s_var_double_ptr = h_double_buf + 3;
        check(s_var_double_ptr == h_double_buf + 3, "heldout_double_ptr_static_local_assign");
        std::printf("CASE 519: heldout_double_ptr_static_local_assign value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 520: heldout_double_ptr_static_local_compound_assign
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        s_var_double_ptr += 1;
        check(s_var_double_ptr == (h_double_buf + 1) + 1, "heldout_double_ptr_static_local_compound_assign");
        std::printf("CASE 520: heldout_double_ptr_static_local_compound_assign value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 521: heldout_double_ptr_static_local_inc_dec
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        ++s_var_double_ptr;
        --s_var_double_ptr;
        check(s_var_double_ptr == h_double_buf + 1, "heldout_double_ptr_static_local_inc_dec");
        std::printf("CASE 521: heldout_double_ptr_static_local_inc_dec value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 522: heldout_double_ptr_static_local_read_after_write
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        s_var_double_ptr = h_double_buf + 3;
        double* read_val = s_var_double_ptr;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_static_local_read_after_write");
        std::printf("CASE 522: heldout_double_ptr_static_local_read_after_write value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 523: heldout_double_ptr_static_local_pass_by_val
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        take_val_h_double_ptr(s_var_double_ptr);
        check(s_var_double_ptr == h_double_buf + 1, "heldout_double_ptr_static_local_pass_by_val");
        std::printf("CASE 523: heldout_double_ptr_static_local_pass_by_val value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 524: heldout_double_ptr_static_local_pass_by_ref
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        take_ref_h_double_ptr(s_var_double_ptr);
        check(s_var_double_ptr == h_double_buf + 1, "heldout_double_ptr_static_local_pass_by_ref");
        std::printf("CASE 524: heldout_double_ptr_static_local_pass_by_ref value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 525: heldout_double_ptr_static_local_return_by_val
    {
        static double* s_var_double_ptr; s_var_double_ptr = h_double_buf + 1;
        s_var_double_ptr = return_h_double_ptr();
        check(s_var_double_ptr == h_double_buf + 3, "heldout_double_ptr_static_local_return_by_val");
        std::printf("CASE 525: heldout_double_ptr_static_local_return_by_val value=%d\n", (s_var_double_ptr != nullptr));
    }

    // Case 526: heldout_double_ptr_struct_member_assign
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        h.m = h_double_buf + 3;
        check(h.m == h_double_buf + 3, "heldout_double_ptr_struct_member_assign");
        std::printf("CASE 526: heldout_double_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 527: heldout_double_ptr_struct_member_compound_assign
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        h.m += 1;
        check(h.m == (h_double_buf + 1) + 1, "heldout_double_ptr_struct_member_compound_assign");
        std::printf("CASE 527: heldout_double_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 528: heldout_double_ptr_struct_member_inc_dec
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        ++h.m;
        --h.m;
        check(h.m == h_double_buf + 1, "heldout_double_ptr_struct_member_inc_dec");
        std::printf("CASE 528: heldout_double_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 529: heldout_double_ptr_struct_member_read_after_write
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        h.m = h_double_buf + 3;
        double* read_val = h.m;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_struct_member_read_after_write");
        std::printf("CASE 529: heldout_double_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 530: heldout_double_ptr_struct_member_pass_by_val
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        take_val_h_double_ptr(h.m);
        check(h.m == h_double_buf + 1, "heldout_double_ptr_struct_member_pass_by_val");
        std::printf("CASE 530: heldout_double_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 531: heldout_double_ptr_struct_member_pass_by_ref
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        take_ref_h_double_ptr(h.m);
        check(h.m == h_double_buf + 1, "heldout_double_ptr_struct_member_pass_by_ref");
        std::printf("CASE 531: heldout_double_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 532: heldout_double_ptr_struct_member_return_by_val
    {
        Holder_h_double_ptr h; h.m = h_double_buf + 1;
        h.m = return_h_double_ptr();
        check(h.m == h_double_buf + 3, "heldout_double_ptr_struct_member_return_by_val");
        std::printf("CASE 532: heldout_double_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 533: heldout_double_ptr_array_elem_assign
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        arr[1] = h_double_buf + 3;
        check(arr[1] == h_double_buf + 3, "heldout_double_ptr_array_elem_assign");
        std::printf("CASE 533: heldout_double_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 534: heldout_double_ptr_array_elem_compound_assign
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        arr[1] += 1;
        check(arr[1] == (h_double_buf + 1) + 1, "heldout_double_ptr_array_elem_compound_assign");
        std::printf("CASE 534: heldout_double_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 535: heldout_double_ptr_array_elem_inc_dec
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        ++arr[1];
        --arr[1];
        check(arr[1] == h_double_buf + 1, "heldout_double_ptr_array_elem_inc_dec");
        std::printf("CASE 535: heldout_double_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 536: heldout_double_ptr_array_elem_read_after_write
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        arr[1] = h_double_buf + 3;
        double* read_val = arr[1];
        check(read_val == h_double_buf + 3, "heldout_double_ptr_array_elem_read_after_write");
        std::printf("CASE 536: heldout_double_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 537: heldout_double_ptr_array_elem_pass_by_val
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        take_val_h_double_ptr(arr[1]);
        check(arr[1] == h_double_buf + 1, "heldout_double_ptr_array_elem_pass_by_val");
        std::printf("CASE 537: heldout_double_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 538: heldout_double_ptr_array_elem_pass_by_ref
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        take_ref_h_double_ptr(arr[1]);
        check(arr[1] == h_double_buf + 1, "heldout_double_ptr_array_elem_pass_by_ref");
        std::printf("CASE 538: heldout_double_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 539: heldout_double_ptr_array_elem_return_by_val
    {
        double* arr[3] = { h_double_buf + 1, h_double_buf + 1, h_double_buf + 1 };
        arr[1] = return_h_double_ptr();
        check(arr[1] == h_double_buf + 3, "heldout_double_ptr_array_elem_return_by_val");
        std::printf("CASE 539: heldout_double_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 540: heldout_double_ptr_via_ref_assign
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        ref = h_double_buf + 3;
        check(ref == h_double_buf + 3, "heldout_double_ptr_via_ref_assign");
        std::printf("CASE 540: heldout_double_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 541: heldout_double_ptr_via_ref_compound_assign
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        ref += 1;
        check(ref == (h_double_buf + 1) + 1, "heldout_double_ptr_via_ref_compound_assign");
        std::printf("CASE 541: heldout_double_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 542: heldout_double_ptr_via_ref_inc_dec
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        ++ref;
        --ref;
        check(ref == h_double_buf + 1, "heldout_double_ptr_via_ref_inc_dec");
        std::printf("CASE 542: heldout_double_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 543: heldout_double_ptr_via_ref_read_after_write
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        ref = h_double_buf + 3;
        double* read_val = ref;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_via_ref_read_after_write");
        std::printf("CASE 543: heldout_double_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 544: heldout_double_ptr_via_ref_pass_by_val
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        take_val_h_double_ptr(ref);
        check(ref == h_double_buf + 1, "heldout_double_ptr_via_ref_pass_by_val");
        std::printf("CASE 544: heldout_double_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 545: heldout_double_ptr_via_ref_pass_by_ref
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        take_ref_h_double_ptr(ref);
        check(ref == h_double_buf + 1, "heldout_double_ptr_via_ref_pass_by_ref");
        std::printf("CASE 545: heldout_double_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 546: heldout_double_ptr_via_ref_return_by_val
    {
        double* base_val = h_double_buf + 1;
        double*& ref = base_val;
        ref = return_h_double_ptr();
        check(ref == h_double_buf + 3, "heldout_double_ptr_via_ref_return_by_val");
        std::printf("CASE 546: heldout_double_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 547: heldout_double_ptr_via_ptr_assign
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        *ptr = h_double_buf + 3;
        check(*ptr == h_double_buf + 3, "heldout_double_ptr_via_ptr_assign");
        std::printf("CASE 547: heldout_double_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 548: heldout_double_ptr_via_ptr_compound_assign
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (h_double_buf + 1) + 1, "heldout_double_ptr_via_ptr_compound_assign");
        std::printf("CASE 548: heldout_double_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 549: heldout_double_ptr_via_ptr_inc_dec
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == h_double_buf + 1, "heldout_double_ptr_via_ptr_inc_dec");
        std::printf("CASE 549: heldout_double_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 550: heldout_double_ptr_via_ptr_read_after_write
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        *ptr = h_double_buf + 3;
        double* read_val = *ptr;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_via_ptr_read_after_write");
        std::printf("CASE 550: heldout_double_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 551: heldout_double_ptr_via_ptr_pass_by_val
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        take_val_h_double_ptr(*ptr);
        check(*ptr == h_double_buf + 1, "heldout_double_ptr_via_ptr_pass_by_val");
        std::printf("CASE 551: heldout_double_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 552: heldout_double_ptr_via_ptr_pass_by_ref
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        take_ref_h_double_ptr(*ptr);
        check(*ptr == h_double_buf + 1, "heldout_double_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 552: heldout_double_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 553: heldout_double_ptr_via_ptr_return_by_val
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        *ptr = return_h_double_ptr();
        check(*ptr == h_double_buf + 3, "heldout_double_ptr_via_ptr_return_by_val");
        std::printf("CASE 553: heldout_double_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 554: heldout_double_ptr_via_pptr_assign
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr = h_double_buf + 3;
        check(**pptr == h_double_buf + 3, "heldout_double_ptr_via_pptr_assign");
        std::printf("CASE 554: heldout_double_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 555: heldout_double_ptr_via_pptr_compound_assign
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (h_double_buf + 1) + 1, "heldout_double_ptr_via_pptr_compound_assign");
        std::printf("CASE 555: heldout_double_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 556: heldout_double_ptr_via_pptr_inc_dec
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == h_double_buf + 1, "heldout_double_ptr_via_pptr_inc_dec");
        std::printf("CASE 556: heldout_double_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 557: heldout_double_ptr_via_pptr_read_after_write
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr = h_double_buf + 3;
        double* read_val = **pptr;
        check(read_val == h_double_buf + 3, "heldout_double_ptr_via_pptr_read_after_write");
        std::printf("CASE 557: heldout_double_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 558: heldout_double_ptr_via_pptr_pass_by_val
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        take_val_h_double_ptr(**pptr);
        check(**pptr == h_double_buf + 1, "heldout_double_ptr_via_pptr_pass_by_val");
        std::printf("CASE 558: heldout_double_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 559: heldout_double_ptr_via_pptr_pass_by_ref
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        take_ref_h_double_ptr(**pptr);
        check(**pptr == h_double_buf + 1, "heldout_double_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 559: heldout_double_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 560: heldout_double_ptr_via_pptr_return_by_val
    {
        double* base_val = h_double_buf + 1;
        double** ptr = &base_val;
        double*** pptr = &ptr;
        **pptr = return_h_double_ptr();
        check(**pptr == h_double_buf + 3, "heldout_double_ptr_via_pptr_return_by_val");
        std::printf("CASE 560: heldout_double_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 561: heldout_struct_ptr_local_assign
    {
        S* lv = h_struct_buf + 1;
        lv = h_struct_buf + 3;
        check(lv == h_struct_buf + 3, "heldout_struct_ptr_local_assign");
        std::printf("CASE 561: heldout_struct_ptr_local_assign value=%d\n", (lv != nullptr));
    }

    // Case 562: heldout_struct_ptr_local_compound_assign
    {
        S* lv = h_struct_buf + 1;
        lv += 1;
        check(lv == (h_struct_buf + 1) + 1, "heldout_struct_ptr_local_compound_assign");
        std::printf("CASE 562: heldout_struct_ptr_local_compound_assign value=%d\n", (lv != nullptr));
    }

    // Case 563: heldout_struct_ptr_local_inc_dec
    {
        S* lv = h_struct_buf + 1;
        ++lv;
        --lv;
        check(lv == h_struct_buf + 1, "heldout_struct_ptr_local_inc_dec");
        std::printf("CASE 563: heldout_struct_ptr_local_inc_dec value=%d\n", (lv != nullptr));
    }

    // Case 564: heldout_struct_ptr_local_read_after_write
    {
        S* lv = h_struct_buf + 1;
        lv = h_struct_buf + 3;
        S* read_val = lv;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_local_read_after_write");
        std::printf("CASE 564: heldout_struct_ptr_local_read_after_write value=%d\n", (lv != nullptr));
    }

    // Case 565: heldout_struct_ptr_local_pass_by_val
    {
        S* lv = h_struct_buf + 1;
        take_val_h_struct_ptr(lv);
        check(lv == h_struct_buf + 1, "heldout_struct_ptr_local_pass_by_val");
        std::printf("CASE 565: heldout_struct_ptr_local_pass_by_val value=%d\n", (lv != nullptr));
    }

    // Case 566: heldout_struct_ptr_local_pass_by_ref
    {
        S* lv = h_struct_buf + 1;
        take_ref_h_struct_ptr(lv);
        check(lv == h_struct_buf + 1, "heldout_struct_ptr_local_pass_by_ref");
        std::printf("CASE 566: heldout_struct_ptr_local_pass_by_ref value=%d\n", (lv != nullptr));
    }

    // Case 567: heldout_struct_ptr_local_return_by_val
    {
        S* lv = h_struct_buf + 1;
        lv = return_h_struct_ptr();
        check(lv == h_struct_buf + 3, "heldout_struct_ptr_local_return_by_val");
        std::printf("CASE 567: heldout_struct_ptr_local_return_by_val value=%d\n", (lv != nullptr));
    }

    // Case 568: heldout_struct_ptr_global_assign
    {
        h_var_struct_ptr = h_struct_buf + 1;
        h_var_struct_ptr = h_struct_buf + 3;
        check(h_var_struct_ptr == h_struct_buf + 3, "heldout_struct_ptr_global_assign");
        std::printf("CASE 568: heldout_struct_ptr_global_assign value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 569: heldout_struct_ptr_global_compound_assign
    {
        h_var_struct_ptr = h_struct_buf + 1;
        h_var_struct_ptr += 1;
        check(h_var_struct_ptr == (h_struct_buf + 1) + 1, "heldout_struct_ptr_global_compound_assign");
        std::printf("CASE 569: heldout_struct_ptr_global_compound_assign value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 570: heldout_struct_ptr_global_inc_dec
    {
        h_var_struct_ptr = h_struct_buf + 1;
        ++h_var_struct_ptr;
        --h_var_struct_ptr;
        check(h_var_struct_ptr == h_struct_buf + 1, "heldout_struct_ptr_global_inc_dec");
        std::printf("CASE 570: heldout_struct_ptr_global_inc_dec value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 571: heldout_struct_ptr_global_read_after_write
    {
        h_var_struct_ptr = h_struct_buf + 1;
        h_var_struct_ptr = h_struct_buf + 3;
        S* read_val = h_var_struct_ptr;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_global_read_after_write");
        std::printf("CASE 571: heldout_struct_ptr_global_read_after_write value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 572: heldout_struct_ptr_global_pass_by_val
    {
        h_var_struct_ptr = h_struct_buf + 1;
        take_val_h_struct_ptr(h_var_struct_ptr);
        check(h_var_struct_ptr == h_struct_buf + 1, "heldout_struct_ptr_global_pass_by_val");
        std::printf("CASE 572: heldout_struct_ptr_global_pass_by_val value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 573: heldout_struct_ptr_global_pass_by_ref
    {
        h_var_struct_ptr = h_struct_buf + 1;
        take_ref_h_struct_ptr(h_var_struct_ptr);
        check(h_var_struct_ptr == h_struct_buf + 1, "heldout_struct_ptr_global_pass_by_ref");
        std::printf("CASE 573: heldout_struct_ptr_global_pass_by_ref value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 574: heldout_struct_ptr_global_return_by_val
    {
        h_var_struct_ptr = h_struct_buf + 1;
        h_var_struct_ptr = return_h_struct_ptr();
        check(h_var_struct_ptr == h_struct_buf + 3, "heldout_struct_ptr_global_return_by_val");
        std::printf("CASE 574: heldout_struct_ptr_global_return_by_val value=%d\n", (h_var_struct_ptr != nullptr));
    }

    // Case 575: heldout_struct_ptr_static_local_assign
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        s_var_struct_ptr = h_struct_buf + 3;
        check(s_var_struct_ptr == h_struct_buf + 3, "heldout_struct_ptr_static_local_assign");
        std::printf("CASE 575: heldout_struct_ptr_static_local_assign value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 576: heldout_struct_ptr_static_local_compound_assign
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        s_var_struct_ptr += 1;
        check(s_var_struct_ptr == (h_struct_buf + 1) + 1, "heldout_struct_ptr_static_local_compound_assign");
        std::printf("CASE 576: heldout_struct_ptr_static_local_compound_assign value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 577: heldout_struct_ptr_static_local_inc_dec
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        ++s_var_struct_ptr;
        --s_var_struct_ptr;
        check(s_var_struct_ptr == h_struct_buf + 1, "heldout_struct_ptr_static_local_inc_dec");
        std::printf("CASE 577: heldout_struct_ptr_static_local_inc_dec value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 578: heldout_struct_ptr_static_local_read_after_write
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        s_var_struct_ptr = h_struct_buf + 3;
        S* read_val = s_var_struct_ptr;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_static_local_read_after_write");
        std::printf("CASE 578: heldout_struct_ptr_static_local_read_after_write value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 579: heldout_struct_ptr_static_local_pass_by_val
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        take_val_h_struct_ptr(s_var_struct_ptr);
        check(s_var_struct_ptr == h_struct_buf + 1, "heldout_struct_ptr_static_local_pass_by_val");
        std::printf("CASE 579: heldout_struct_ptr_static_local_pass_by_val value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 580: heldout_struct_ptr_static_local_pass_by_ref
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        take_ref_h_struct_ptr(s_var_struct_ptr);
        check(s_var_struct_ptr == h_struct_buf + 1, "heldout_struct_ptr_static_local_pass_by_ref");
        std::printf("CASE 580: heldout_struct_ptr_static_local_pass_by_ref value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 581: heldout_struct_ptr_static_local_return_by_val
    {
        static S* s_var_struct_ptr; s_var_struct_ptr = h_struct_buf + 1;
        s_var_struct_ptr = return_h_struct_ptr();
        check(s_var_struct_ptr == h_struct_buf + 3, "heldout_struct_ptr_static_local_return_by_val");
        std::printf("CASE 581: heldout_struct_ptr_static_local_return_by_val value=%d\n", (s_var_struct_ptr != nullptr));
    }

    // Case 582: heldout_struct_ptr_struct_member_assign
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        h.m = h_struct_buf + 3;
        check(h.m == h_struct_buf + 3, "heldout_struct_ptr_struct_member_assign");
        std::printf("CASE 582: heldout_struct_ptr_struct_member_assign value=%d\n", (h.m != nullptr));
    }

    // Case 583: heldout_struct_ptr_struct_member_compound_assign
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        h.m += 1;
        check(h.m == (h_struct_buf + 1) + 1, "heldout_struct_ptr_struct_member_compound_assign");
        std::printf("CASE 583: heldout_struct_ptr_struct_member_compound_assign value=%d\n", (h.m != nullptr));
    }

    // Case 584: heldout_struct_ptr_struct_member_inc_dec
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        ++h.m;
        --h.m;
        check(h.m == h_struct_buf + 1, "heldout_struct_ptr_struct_member_inc_dec");
        std::printf("CASE 584: heldout_struct_ptr_struct_member_inc_dec value=%d\n", (h.m != nullptr));
    }

    // Case 585: heldout_struct_ptr_struct_member_read_after_write
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        h.m = h_struct_buf + 3;
        S* read_val = h.m;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_struct_member_read_after_write");
        std::printf("CASE 585: heldout_struct_ptr_struct_member_read_after_write value=%d\n", (h.m != nullptr));
    }

    // Case 586: heldout_struct_ptr_struct_member_pass_by_val
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        take_val_h_struct_ptr(h.m);
        check(h.m == h_struct_buf + 1, "heldout_struct_ptr_struct_member_pass_by_val");
        std::printf("CASE 586: heldout_struct_ptr_struct_member_pass_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 587: heldout_struct_ptr_struct_member_pass_by_ref
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        take_ref_h_struct_ptr(h.m);
        check(h.m == h_struct_buf + 1, "heldout_struct_ptr_struct_member_pass_by_ref");
        std::printf("CASE 587: heldout_struct_ptr_struct_member_pass_by_ref value=%d\n", (h.m != nullptr));
    }

    // Case 588: heldout_struct_ptr_struct_member_return_by_val
    {
        Holder_h_struct_ptr h; h.m = h_struct_buf + 1;
        h.m = return_h_struct_ptr();
        check(h.m == h_struct_buf + 3, "heldout_struct_ptr_struct_member_return_by_val");
        std::printf("CASE 588: heldout_struct_ptr_struct_member_return_by_val value=%d\n", (h.m != nullptr));
    }

    // Case 589: heldout_struct_ptr_array_elem_assign
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        arr[1] = h_struct_buf + 3;
        check(arr[1] == h_struct_buf + 3, "heldout_struct_ptr_array_elem_assign");
        std::printf("CASE 589: heldout_struct_ptr_array_elem_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 590: heldout_struct_ptr_array_elem_compound_assign
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        arr[1] += 1;
        check(arr[1] == (h_struct_buf + 1) + 1, "heldout_struct_ptr_array_elem_compound_assign");
        std::printf("CASE 590: heldout_struct_ptr_array_elem_compound_assign value=%d\n", (arr[1] != nullptr));
    }

    // Case 591: heldout_struct_ptr_array_elem_inc_dec
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        ++arr[1];
        --arr[1];
        check(arr[1] == h_struct_buf + 1, "heldout_struct_ptr_array_elem_inc_dec");
        std::printf("CASE 591: heldout_struct_ptr_array_elem_inc_dec value=%d\n", (arr[1] != nullptr));
    }

    // Case 592: heldout_struct_ptr_array_elem_read_after_write
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        arr[1] = h_struct_buf + 3;
        S* read_val = arr[1];
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_array_elem_read_after_write");
        std::printf("CASE 592: heldout_struct_ptr_array_elem_read_after_write value=%d\n", (arr[1] != nullptr));
    }

    // Case 593: heldout_struct_ptr_array_elem_pass_by_val
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        take_val_h_struct_ptr(arr[1]);
        check(arr[1] == h_struct_buf + 1, "heldout_struct_ptr_array_elem_pass_by_val");
        std::printf("CASE 593: heldout_struct_ptr_array_elem_pass_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 594: heldout_struct_ptr_array_elem_pass_by_ref
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        take_ref_h_struct_ptr(arr[1]);
        check(arr[1] == h_struct_buf + 1, "heldout_struct_ptr_array_elem_pass_by_ref");
        std::printf("CASE 594: heldout_struct_ptr_array_elem_pass_by_ref value=%d\n", (arr[1] != nullptr));
    }

    // Case 595: heldout_struct_ptr_array_elem_return_by_val
    {
        S* arr[3] = { h_struct_buf + 1, h_struct_buf + 1, h_struct_buf + 1 };
        arr[1] = return_h_struct_ptr();
        check(arr[1] == h_struct_buf + 3, "heldout_struct_ptr_array_elem_return_by_val");
        std::printf("CASE 595: heldout_struct_ptr_array_elem_return_by_val value=%d\n", (arr[1] != nullptr));
    }

    // Case 596: heldout_struct_ptr_via_ref_assign
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        ref = h_struct_buf + 3;
        check(ref == h_struct_buf + 3, "heldout_struct_ptr_via_ref_assign");
        std::printf("CASE 596: heldout_struct_ptr_via_ref_assign value=%d\n", (ref != nullptr));
    }

    // Case 597: heldout_struct_ptr_via_ref_compound_assign
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        ref += 1;
        check(ref == (h_struct_buf + 1) + 1, "heldout_struct_ptr_via_ref_compound_assign");
        std::printf("CASE 597: heldout_struct_ptr_via_ref_compound_assign value=%d\n", (ref != nullptr));
    }

    // Case 598: heldout_struct_ptr_via_ref_inc_dec
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        ++ref;
        --ref;
        check(ref == h_struct_buf + 1, "heldout_struct_ptr_via_ref_inc_dec");
        std::printf("CASE 598: heldout_struct_ptr_via_ref_inc_dec value=%d\n", (ref != nullptr));
    }

    // Case 599: heldout_struct_ptr_via_ref_read_after_write
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        ref = h_struct_buf + 3;
        S* read_val = ref;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_via_ref_read_after_write");
        std::printf("CASE 599: heldout_struct_ptr_via_ref_read_after_write value=%d\n", (ref != nullptr));
    }

    // Case 600: heldout_struct_ptr_via_ref_pass_by_val
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        take_val_h_struct_ptr(ref);
        check(ref == h_struct_buf + 1, "heldout_struct_ptr_via_ref_pass_by_val");
        std::printf("CASE 600: heldout_struct_ptr_via_ref_pass_by_val value=%d\n", (ref != nullptr));
    }

    // Case 601: heldout_struct_ptr_via_ref_pass_by_ref
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        take_ref_h_struct_ptr(ref);
        check(ref == h_struct_buf + 1, "heldout_struct_ptr_via_ref_pass_by_ref");
        std::printf("CASE 601: heldout_struct_ptr_via_ref_pass_by_ref value=%d\n", (ref != nullptr));
    }

    // Case 602: heldout_struct_ptr_via_ref_return_by_val
    {
        S* base_val = h_struct_buf + 1;
        S*& ref = base_val;
        ref = return_h_struct_ptr();
        check(ref == h_struct_buf + 3, "heldout_struct_ptr_via_ref_return_by_val");
        std::printf("CASE 602: heldout_struct_ptr_via_ref_return_by_val value=%d\n", (ref != nullptr));
    }

    // Case 603: heldout_struct_ptr_via_ptr_assign
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        *ptr = h_struct_buf + 3;
        check(*ptr == h_struct_buf + 3, "heldout_struct_ptr_via_ptr_assign");
        std::printf("CASE 603: heldout_struct_ptr_via_ptr_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 604: heldout_struct_ptr_via_ptr_compound_assign
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        *ptr += 1;
        check(*ptr == (h_struct_buf + 1) + 1, "heldout_struct_ptr_via_ptr_compound_assign");
        std::printf("CASE 604: heldout_struct_ptr_via_ptr_compound_assign value=%d\n", (*ptr != nullptr));
    }

    // Case 605: heldout_struct_ptr_via_ptr_inc_dec
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        ++*ptr;
        --*ptr;
        check(*ptr == h_struct_buf + 1, "heldout_struct_ptr_via_ptr_inc_dec");
        std::printf("CASE 605: heldout_struct_ptr_via_ptr_inc_dec value=%d\n", (*ptr != nullptr));
    }

    // Case 606: heldout_struct_ptr_via_ptr_read_after_write
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        *ptr = h_struct_buf + 3;
        S* read_val = *ptr;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_via_ptr_read_after_write");
        std::printf("CASE 606: heldout_struct_ptr_via_ptr_read_after_write value=%d\n", (*ptr != nullptr));
    }

    // Case 607: heldout_struct_ptr_via_ptr_pass_by_val
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        take_val_h_struct_ptr(*ptr);
        check(*ptr == h_struct_buf + 1, "heldout_struct_ptr_via_ptr_pass_by_val");
        std::printf("CASE 607: heldout_struct_ptr_via_ptr_pass_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 608: heldout_struct_ptr_via_ptr_pass_by_ref
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        take_ref_h_struct_ptr(*ptr);
        check(*ptr == h_struct_buf + 1, "heldout_struct_ptr_via_ptr_pass_by_ref");
        std::printf("CASE 608: heldout_struct_ptr_via_ptr_pass_by_ref value=%d\n", (*ptr != nullptr));
    }

    // Case 609: heldout_struct_ptr_via_ptr_return_by_val
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        *ptr = return_h_struct_ptr();
        check(*ptr == h_struct_buf + 3, "heldout_struct_ptr_via_ptr_return_by_val");
        std::printf("CASE 609: heldout_struct_ptr_via_ptr_return_by_val value=%d\n", (*ptr != nullptr));
    }

    // Case 610: heldout_struct_ptr_via_pptr_assign
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr = h_struct_buf + 3;
        check(**pptr == h_struct_buf + 3, "heldout_struct_ptr_via_pptr_assign");
        std::printf("CASE 610: heldout_struct_ptr_via_pptr_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 611: heldout_struct_ptr_via_pptr_compound_assign
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr += 1;
        check(**pptr == (h_struct_buf + 1) + 1, "heldout_struct_ptr_via_pptr_compound_assign");
        std::printf("CASE 611: heldout_struct_ptr_via_pptr_compound_assign value=%d\n", (**pptr != nullptr));
    }

    // Case 612: heldout_struct_ptr_via_pptr_inc_dec
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        ++**pptr;
        --**pptr;
        check(**pptr == h_struct_buf + 1, "heldout_struct_ptr_via_pptr_inc_dec");
        std::printf("CASE 612: heldout_struct_ptr_via_pptr_inc_dec value=%d\n", (**pptr != nullptr));
    }

    // Case 613: heldout_struct_ptr_via_pptr_read_after_write
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr = h_struct_buf + 3;
        S* read_val = **pptr;
        check(read_val == h_struct_buf + 3, "heldout_struct_ptr_via_pptr_read_after_write");
        std::printf("CASE 613: heldout_struct_ptr_via_pptr_read_after_write value=%d\n", (**pptr != nullptr));
    }

    // Case 614: heldout_struct_ptr_via_pptr_pass_by_val
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        take_val_h_struct_ptr(**pptr);
        check(**pptr == h_struct_buf + 1, "heldout_struct_ptr_via_pptr_pass_by_val");
        std::printf("CASE 614: heldout_struct_ptr_via_pptr_pass_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 615: heldout_struct_ptr_via_pptr_pass_by_ref
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        take_ref_h_struct_ptr(**pptr);
        check(**pptr == h_struct_buf + 1, "heldout_struct_ptr_via_pptr_pass_by_ref");
        std::printf("CASE 615: heldout_struct_ptr_via_pptr_pass_by_ref value=%d\n", (**pptr != nullptr));
    }

    // Case 616: heldout_struct_ptr_via_pptr_return_by_val
    {
        S* base_val = h_struct_buf + 1;
        S** ptr = &base_val;
        S*** pptr = &ptr;
        **pptr = return_h_struct_ptr();
        check(**pptr == h_struct_buf + 3, "heldout_struct_ptr_via_pptr_return_by_val");
        std::printf("CASE 616: heldout_struct_ptr_via_pptr_return_by_val value=%d\n", (**pptr != nullptr));
    }

    // Case 617: heldout_enum_color_local_assign
    {
        Color lv = Red;
        lv = Blue;
        check(lv == Blue, "heldout_enum_color_local_assign");
        std::printf("CASE 617: heldout_enum_color_local_assign value=%d\n", (int)(lv));
    }

    // Case 618: heldout_enum_color_local_compound_assign
    {
        Color lv = Red;
        lv = Blue;
        check(lv == Blue, "heldout_enum_color_local_compound_assign");
        std::printf("CASE 618: heldout_enum_color_local_compound_assign value=%d\n", (int)(lv));
    }

    // Case 619: heldout_enum_color_local_inc_dec
    {
        Color lv = Red;
        lv = Red;
        check(lv == Red, "heldout_enum_color_local_inc_dec");
        std::printf("CASE 619: heldout_enum_color_local_inc_dec value=%d\n", (int)(lv));
    }

    // Case 620: heldout_enum_color_local_read_after_write
    {
        Color lv = Red;
        lv = Blue;
        Color read_val = lv;
        check(read_val == Blue, "heldout_enum_color_local_read_after_write");
        std::printf("CASE 620: heldout_enum_color_local_read_after_write value=%d\n", (int)(lv));
    }

    // Case 621: heldout_enum_color_local_pass_by_val
    {
        Color lv = Red;
        take_val_h_enum_color(lv);
        check(lv == Red, "heldout_enum_color_local_pass_by_val");
        std::printf("CASE 621: heldout_enum_color_local_pass_by_val value=%d\n", (int)(lv));
    }

    // Case 622: heldout_enum_color_local_pass_by_ref
    {
        Color lv = Red;
        take_ref_h_enum_color(lv);
        check(lv == Red, "heldout_enum_color_local_pass_by_ref");
        std::printf("CASE 622: heldout_enum_color_local_pass_by_ref value=%d\n", (int)(lv));
    }

    // Case 623: heldout_enum_color_local_return_by_val
    {
        Color lv = Red;
        lv = return_h_enum_color();
        check(lv == Blue, "heldout_enum_color_local_return_by_val");
        std::printf("CASE 623: heldout_enum_color_local_return_by_val value=%d\n", (int)(lv));
    }

    // Case 624: heldout_enum_color_global_assign
    {
        h_var_enum_color = Red;
        h_var_enum_color = Blue;
        check(h_var_enum_color == Blue, "heldout_enum_color_global_assign");
        std::printf("CASE 624: heldout_enum_color_global_assign value=%d\n", (int)(h_var_enum_color));
    }

    // Case 625: heldout_enum_color_global_compound_assign
    {
        h_var_enum_color = Red;
        h_var_enum_color = Blue;
        check(h_var_enum_color == Blue, "heldout_enum_color_global_compound_assign");
        std::printf("CASE 625: heldout_enum_color_global_compound_assign value=%d\n", (int)(h_var_enum_color));
    }

    // Case 626: heldout_enum_color_global_inc_dec
    {
        h_var_enum_color = Red;
        h_var_enum_color = Red;
        check(h_var_enum_color == Red, "heldout_enum_color_global_inc_dec");
        std::printf("CASE 626: heldout_enum_color_global_inc_dec value=%d\n", (int)(h_var_enum_color));
    }

    // Case 627: heldout_enum_color_global_read_after_write
    {
        h_var_enum_color = Red;
        h_var_enum_color = Blue;
        Color read_val = h_var_enum_color;
        check(read_val == Blue, "heldout_enum_color_global_read_after_write");
        std::printf("CASE 627: heldout_enum_color_global_read_after_write value=%d\n", (int)(h_var_enum_color));
    }

    // Case 628: heldout_enum_color_global_pass_by_val
    {
        h_var_enum_color = Red;
        take_val_h_enum_color(h_var_enum_color);
        check(h_var_enum_color == Red, "heldout_enum_color_global_pass_by_val");
        std::printf("CASE 628: heldout_enum_color_global_pass_by_val value=%d\n", (int)(h_var_enum_color));
    }

    // Case 629: heldout_enum_color_global_pass_by_ref
    {
        h_var_enum_color = Red;
        take_ref_h_enum_color(h_var_enum_color);
        check(h_var_enum_color == Red, "heldout_enum_color_global_pass_by_ref");
        std::printf("CASE 629: heldout_enum_color_global_pass_by_ref value=%d\n", (int)(h_var_enum_color));
    }

    // Case 630: heldout_enum_color_global_return_by_val
    {
        h_var_enum_color = Red;
        h_var_enum_color = return_h_enum_color();
        check(h_var_enum_color == Blue, "heldout_enum_color_global_return_by_val");
        std::printf("CASE 630: heldout_enum_color_global_return_by_val value=%d\n", (int)(h_var_enum_color));
    }

    // Case 631: heldout_enum_color_static_local_assign
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        s_var_enum_color = Blue;
        check(s_var_enum_color == Blue, "heldout_enum_color_static_local_assign");
        std::printf("CASE 631: heldout_enum_color_static_local_assign value=%d\n", (int)(s_var_enum_color));
    }

    // Case 632: heldout_enum_color_static_local_compound_assign
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        s_var_enum_color = Blue;
        check(s_var_enum_color == Blue, "heldout_enum_color_static_local_compound_assign");
        std::printf("CASE 632: heldout_enum_color_static_local_compound_assign value=%d\n", (int)(s_var_enum_color));
    }

    // Case 633: heldout_enum_color_static_local_inc_dec
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        s_var_enum_color = Red;
        check(s_var_enum_color == Red, "heldout_enum_color_static_local_inc_dec");
        std::printf("CASE 633: heldout_enum_color_static_local_inc_dec value=%d\n", (int)(s_var_enum_color));
    }

    // Case 634: heldout_enum_color_static_local_read_after_write
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        s_var_enum_color = Blue;
        Color read_val = s_var_enum_color;
        check(read_val == Blue, "heldout_enum_color_static_local_read_after_write");
        std::printf("CASE 634: heldout_enum_color_static_local_read_after_write value=%d\n", (int)(s_var_enum_color));
    }

    // Case 635: heldout_enum_color_static_local_pass_by_val
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        take_val_h_enum_color(s_var_enum_color);
        check(s_var_enum_color == Red, "heldout_enum_color_static_local_pass_by_val");
        std::printf("CASE 635: heldout_enum_color_static_local_pass_by_val value=%d\n", (int)(s_var_enum_color));
    }

    // Case 636: heldout_enum_color_static_local_pass_by_ref
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        take_ref_h_enum_color(s_var_enum_color);
        check(s_var_enum_color == Red, "heldout_enum_color_static_local_pass_by_ref");
        std::printf("CASE 636: heldout_enum_color_static_local_pass_by_ref value=%d\n", (int)(s_var_enum_color));
    }

    // Case 637: heldout_enum_color_static_local_return_by_val
    {
        static Color s_var_enum_color; s_var_enum_color = Red;
        s_var_enum_color = return_h_enum_color();
        check(s_var_enum_color == Blue, "heldout_enum_color_static_local_return_by_val");
        std::printf("CASE 637: heldout_enum_color_static_local_return_by_val value=%d\n", (int)(s_var_enum_color));
    }

    // Case 638: heldout_enum_color_struct_member_assign
    {
        Holder_h_enum_color h; h.m = Red;
        h.m = Blue;
        check(h.m == Blue, "heldout_enum_color_struct_member_assign");
        std::printf("CASE 638: heldout_enum_color_struct_member_assign value=%d\n", (int)(h.m));
    }

    // Case 639: heldout_enum_color_struct_member_compound_assign
    {
        Holder_h_enum_color h; h.m = Red;
        h.m = Blue;
        check(h.m == Blue, "heldout_enum_color_struct_member_compound_assign");
        std::printf("CASE 639: heldout_enum_color_struct_member_compound_assign value=%d\n", (int)(h.m));
    }

    // Case 640: heldout_enum_color_struct_member_inc_dec
    {
        Holder_h_enum_color h; h.m = Red;
        h.m = Red;
        check(h.m == Red, "heldout_enum_color_struct_member_inc_dec");
        std::printf("CASE 640: heldout_enum_color_struct_member_inc_dec value=%d\n", (int)(h.m));
    }

    // Case 641: heldout_enum_color_struct_member_read_after_write
    {
        Holder_h_enum_color h; h.m = Red;
        h.m = Blue;
        Color read_val = h.m;
        check(read_val == Blue, "heldout_enum_color_struct_member_read_after_write");
        std::printf("CASE 641: heldout_enum_color_struct_member_read_after_write value=%d\n", (int)(h.m));
    }

    // Case 642: heldout_enum_color_struct_member_pass_by_val
    {
        Holder_h_enum_color h; h.m = Red;
        take_val_h_enum_color(h.m);
        check(h.m == Red, "heldout_enum_color_struct_member_pass_by_val");
        std::printf("CASE 642: heldout_enum_color_struct_member_pass_by_val value=%d\n", (int)(h.m));
    }

    // Case 643: heldout_enum_color_struct_member_pass_by_ref
    {
        Holder_h_enum_color h; h.m = Red;
        take_ref_h_enum_color(h.m);
        check(h.m == Red, "heldout_enum_color_struct_member_pass_by_ref");
        std::printf("CASE 643: heldout_enum_color_struct_member_pass_by_ref value=%d\n", (int)(h.m));
    }

    // Case 644: heldout_enum_color_struct_member_return_by_val
    {
        Holder_h_enum_color h; h.m = Red;
        h.m = return_h_enum_color();
        check(h.m == Blue, "heldout_enum_color_struct_member_return_by_val");
        std::printf("CASE 644: heldout_enum_color_struct_member_return_by_val value=%d\n", (int)(h.m));
    }

    // Case 645: heldout_enum_color_array_elem_assign
    {
        Color arr[3] = { Red, Red, Red };
        arr[1] = Blue;
        check(arr[1] == Blue, "heldout_enum_color_array_elem_assign");
        std::printf("CASE 645: heldout_enum_color_array_elem_assign value=%d\n", (int)(arr[1]));
    }

    // Case 646: heldout_enum_color_array_elem_compound_assign
    {
        Color arr[3] = { Red, Red, Red };
        arr[1] = Blue;
        check(arr[1] == Blue, "heldout_enum_color_array_elem_compound_assign");
        std::printf("CASE 646: heldout_enum_color_array_elem_compound_assign value=%d\n", (int)(arr[1]));
    }

    // Case 647: heldout_enum_color_array_elem_inc_dec
    {
        Color arr[3] = { Red, Red, Red };
        arr[1] = Red;
        check(arr[1] == Red, "heldout_enum_color_array_elem_inc_dec");
        std::printf("CASE 647: heldout_enum_color_array_elem_inc_dec value=%d\n", (int)(arr[1]));
    }

    // Case 648: heldout_enum_color_array_elem_read_after_write
    {
        Color arr[3] = { Red, Red, Red };
        arr[1] = Blue;
        Color read_val = arr[1];
        check(read_val == Blue, "heldout_enum_color_array_elem_read_after_write");
        std::printf("CASE 648: heldout_enum_color_array_elem_read_after_write value=%d\n", (int)(arr[1]));
    }

    // Case 649: heldout_enum_color_array_elem_pass_by_val
    {
        Color arr[3] = { Red, Red, Red };
        take_val_h_enum_color(arr[1]);
        check(arr[1] == Red, "heldout_enum_color_array_elem_pass_by_val");
        std::printf("CASE 649: heldout_enum_color_array_elem_pass_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 650: heldout_enum_color_array_elem_pass_by_ref
    {
        Color arr[3] = { Red, Red, Red };
        take_ref_h_enum_color(arr[1]);
        check(arr[1] == Red, "heldout_enum_color_array_elem_pass_by_ref");
        std::printf("CASE 650: heldout_enum_color_array_elem_pass_by_ref value=%d\n", (int)(arr[1]));
    }

    // Case 651: heldout_enum_color_array_elem_return_by_val
    {
        Color arr[3] = { Red, Red, Red };
        arr[1] = return_h_enum_color();
        check(arr[1] == Blue, "heldout_enum_color_array_elem_return_by_val");
        std::printf("CASE 651: heldout_enum_color_array_elem_return_by_val value=%d\n", (int)(arr[1]));
    }

    // Case 652: heldout_enum_color_via_ref_assign
    {
        Color base_val = Red;
        Color& ref = base_val;
        ref = Blue;
        check(ref == Blue, "heldout_enum_color_via_ref_assign");
        std::printf("CASE 652: heldout_enum_color_via_ref_assign value=%d\n", (int)(ref));
    }

    // Case 653: heldout_enum_color_via_ref_compound_assign
    {
        Color base_val = Red;
        Color& ref = base_val;
        ref = Blue;
        check(ref == Blue, "heldout_enum_color_via_ref_compound_assign");
        std::printf("CASE 653: heldout_enum_color_via_ref_compound_assign value=%d\n", (int)(ref));
    }

    // Case 654: heldout_enum_color_via_ref_inc_dec
    {
        Color base_val = Red;
        Color& ref = base_val;
        ref = Red;
        check(ref == Red, "heldout_enum_color_via_ref_inc_dec");
        std::printf("CASE 654: heldout_enum_color_via_ref_inc_dec value=%d\n", (int)(ref));
    }

    // Case 655: heldout_enum_color_via_ref_read_after_write
    {
        Color base_val = Red;
        Color& ref = base_val;
        ref = Blue;
        Color read_val = ref;
        check(read_val == Blue, "heldout_enum_color_via_ref_read_after_write");
        std::printf("CASE 655: heldout_enum_color_via_ref_read_after_write value=%d\n", (int)(ref));
    }

    // Case 656: heldout_enum_color_via_ref_pass_by_val
    {
        Color base_val = Red;
        Color& ref = base_val;
        take_val_h_enum_color(ref);
        check(ref == Red, "heldout_enum_color_via_ref_pass_by_val");
        std::printf("CASE 656: heldout_enum_color_via_ref_pass_by_val value=%d\n", (int)(ref));
    }

    // Case 657: heldout_enum_color_via_ref_pass_by_ref
    {
        Color base_val = Red;
        Color& ref = base_val;
        take_ref_h_enum_color(ref);
        check(ref == Red, "heldout_enum_color_via_ref_pass_by_ref");
        std::printf("CASE 657: heldout_enum_color_via_ref_pass_by_ref value=%d\n", (int)(ref));
    }

    // Case 658: heldout_enum_color_via_ref_return_by_val
    {
        Color base_val = Red;
        Color& ref = base_val;
        ref = return_h_enum_color();
        check(ref == Blue, "heldout_enum_color_via_ref_return_by_val");
        std::printf("CASE 658: heldout_enum_color_via_ref_return_by_val value=%d\n", (int)(ref));
    }

    // Case 659: heldout_enum_color_via_ptr_assign
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        *ptr = Blue;
        check(*ptr == Blue, "heldout_enum_color_via_ptr_assign");
        std::printf("CASE 659: heldout_enum_color_via_ptr_assign value=%d\n", (int)(*ptr));
    }

    // Case 660: heldout_enum_color_via_ptr_compound_assign
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        *ptr = Blue;
        check(*ptr == Blue, "heldout_enum_color_via_ptr_compound_assign");
        std::printf("CASE 660: heldout_enum_color_via_ptr_compound_assign value=%d\n", (int)(*ptr));
    }

    // Case 661: heldout_enum_color_via_ptr_inc_dec
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        *ptr = Red;
        check(*ptr == Red, "heldout_enum_color_via_ptr_inc_dec");
        std::printf("CASE 661: heldout_enum_color_via_ptr_inc_dec value=%d\n", (int)(*ptr));
    }

    // Case 662: heldout_enum_color_via_ptr_read_after_write
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        *ptr = Blue;
        Color read_val = *ptr;
        check(read_val == Blue, "heldout_enum_color_via_ptr_read_after_write");
        std::printf("CASE 662: heldout_enum_color_via_ptr_read_after_write value=%d\n", (int)(*ptr));
    }

    // Case 663: heldout_enum_color_via_ptr_pass_by_val
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        take_val_h_enum_color(*ptr);
        check(*ptr == Red, "heldout_enum_color_via_ptr_pass_by_val");
        std::printf("CASE 663: heldout_enum_color_via_ptr_pass_by_val value=%d\n", (int)(*ptr));
    }

    // Case 664: heldout_enum_color_via_ptr_pass_by_ref
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        take_ref_h_enum_color(*ptr);
        check(*ptr == Red, "heldout_enum_color_via_ptr_pass_by_ref");
        std::printf("CASE 664: heldout_enum_color_via_ptr_pass_by_ref value=%d\n", (int)(*ptr));
    }

    // Case 665: heldout_enum_color_via_ptr_return_by_val
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        *ptr = return_h_enum_color();
        check(*ptr == Blue, "heldout_enum_color_via_ptr_return_by_val");
        std::printf("CASE 665: heldout_enum_color_via_ptr_return_by_val value=%d\n", (int)(*ptr));
    }

    // Case 666: heldout_enum_color_via_pptr_assign
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Blue;
        check(**pptr == Blue, "heldout_enum_color_via_pptr_assign");
        std::printf("CASE 666: heldout_enum_color_via_pptr_assign value=%d\n", (int)(**pptr));
    }

    // Case 667: heldout_enum_color_via_pptr_compound_assign
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Blue;
        check(**pptr == Blue, "heldout_enum_color_via_pptr_compound_assign");
        std::printf("CASE 667: heldout_enum_color_via_pptr_compound_assign value=%d\n", (int)(**pptr));
    }

    // Case 668: heldout_enum_color_via_pptr_inc_dec
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Red;
        check(**pptr == Red, "heldout_enum_color_via_pptr_inc_dec");
        std::printf("CASE 668: heldout_enum_color_via_pptr_inc_dec value=%d\n", (int)(**pptr));
    }

    // Case 669: heldout_enum_color_via_pptr_read_after_write
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = Blue;
        Color read_val = **pptr;
        check(read_val == Blue, "heldout_enum_color_via_pptr_read_after_write");
        std::printf("CASE 669: heldout_enum_color_via_pptr_read_after_write value=%d\n", (int)(**pptr));
    }

    // Case 670: heldout_enum_color_via_pptr_pass_by_val
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        take_val_h_enum_color(**pptr);
        check(**pptr == Red, "heldout_enum_color_via_pptr_pass_by_val");
        std::printf("CASE 670: heldout_enum_color_via_pptr_pass_by_val value=%d\n", (int)(**pptr));
    }

    // Case 671: heldout_enum_color_via_pptr_pass_by_ref
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        take_ref_h_enum_color(**pptr);
        check(**pptr == Red, "heldout_enum_color_via_pptr_pass_by_ref");
        std::printf("CASE 671: heldout_enum_color_via_pptr_pass_by_ref value=%d\n", (int)(**pptr));
    }

    // Case 672: heldout_enum_color_via_pptr_return_by_val
    {
        Color base_val = Red;
        Color* ptr = &base_val;
        Color** pptr = &ptr;
        **pptr = return_h_enum_color();
        check(**pptr == Blue, "heldout_enum_color_via_pptr_return_by_val");
        std::printf("CASE 672: heldout_enum_color_via_pptr_return_by_val value=%d\n", (int)(**pptr));
    }

    std::printf("Total test cases executed: %d, verified checks: %d, rolling hash: %u\n", 672, total_checks, rolling_hash);
    return 0;
}