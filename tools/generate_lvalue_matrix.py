#!/usr/bin/env python3
import subprocess
import sys
import os

TYPES_MAIN = [
    ("char", "char", "'A'", "'B'", 1, True, True),
    ("short", "short", "12", "34", 5, True, True),
    ("int", "int", "101", "202", 7, True, True),
    ("long_long", "long long", "1000000000LL", "2000000000LL", 100, True, True),
    ("float", "float", "1.5f", "4.5f", 2.0, True, True),
    ("double", "double", "2.25", "6.75", 3.0, True, True),
    ("bool", "bool", "true", "false", 0, False, False),
    ("char_ptr", "char*", "g_char_buf + 0", "g_char_buf + 2", 1, True, True),
    ("int_ptr", "int*", "g_int_buf + 0", "g_int_buf + 2", 1, True, True),
    ("double_ptr", "double*", "g_double_buf + 0", "g_double_buf + 2", 1, True, True),
    ("struct_ptr", "S*", "g_struct_buf + 0", "g_struct_buf + 2", 1, True, True),
    ("enum_color", "Color", "Green", "Blue", 0, False, False),
]

TYPES_HELDOUT = [
    ("char", "char", "'C'", "'D'", 1, True, True),
    ("short", "short", "20", "40", 6, True, True),
    ("int", "int", "301", "402", 9, True, True),
    ("long_long", "long long", "3000000000LL", "4000000000LL", 150, True, True),
    ("float", "float", "2.5f", "6.5f", 3.0, True, True),
    ("double", "double", "3.25", "7.75", 4.0, True, True),
    ("bool", "bool", "false", "true", 0, False, False),
    ("char_ptr", "char*", "h_char_buf + 1", "h_char_buf + 3", 1, True, True),
    ("int_ptr", "int*", "h_int_buf + 1", "h_int_buf + 3", 1, True, True),
    ("double_ptr", "double*", "h_double_buf + 1", "h_double_buf + 3", 1, True, True),
    ("struct_ptr", "S*", "h_struct_buf + 1", "h_struct_buf + 3", 1, True, True),
    ("enum_color", "Color", "Red", "Blue", 0, False, False),
]

LVALUE_KINDS = [
    "local",
    "global",
    "static_local",
    "struct_member",
    "array_elem",
    "via_ref",
    "via_ptr",
    "via_pptr",
]

OPERATIONS = [
    "assign",
    "compound_assign",
    "inc_dec",
    "read_after_write",
    "pass_by_val",
    "pass_by_ref",
    "return_by_val",
]

def generate_cpp(is_heldout=False):
    prefix_pfx = "h" if is_heldout else "g"
    types = TYPES_HELDOUT if is_heldout else TYPES_MAIN
    lines = []
    lines.append("#include <cstdio>")
    lines.append("")
    lines.append("enum Color { Red = 0, Green = 1, Blue = 2 };")
    lines.append("struct S { int id; };")
    lines.append("")
    if is_heldout:
        lines.append("char h_char_buf[16] = { 'z', 'y', 'x', 'w', 'v', 'u', 't', 's', 'r', 'q', 'p', 'o', 'n', 'm', 'l', 0 };")
        lines.append("int h_int_buf[8] = {100, 200, 300, 400, 500, 600, 700, 800};")
        lines.append("double h_double_buf[8] = {1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5};")
        lines.append("S h_struct_buf[8] = {{10}, {20}, {30}, {40}, {50}, {60}, {70}, {80}};")
    else:
        lines.append("char g_char_buf[16] = { 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 0 };")
        lines.append("int g_int_buf[8] = {10, 20, 30, 40, 50, 60, 70, 80};")
        lines.append("double g_double_buf[8] = {1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8};")
        lines.append("S g_struct_buf[8] = {{1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}};")
    lines.append("")
    
    # Global variables for each type
    for name, ty, v1, v2, delta, has_comp, has_inc in types:
        lines.append(f"{ty} {prefix_pfx}_var_{name} = {v1};")
        lines.append(f"struct Holder_{prefix_pfx}_{name} {{ {ty} m; }};")
        lines.append(f"Holder_{prefix_pfx}_{name} {prefix_pfx}_holder_{name} = {{ {v1} }};")
        lines.append(f"{ty} {prefix_pfx}_arr_{name}[4] = {{ {v1}, {v1}, {v2}, {v2} }};")
        lines.append(f"void take_val_{prefix_pfx}_{name}({ty} x) {{ (void)x; }}")
        lines.append(f"void take_ref_{prefix_pfx}_{name}({ty}& x) {{ (void)x; }}")
        lines.append(f"{ty} return_{prefix_pfx}_{name}() {{ return {v2}; }}")
        lines.append("")

    lines.append("int total_checks = 0;")
    hash_seed = "2654435761u" if is_heldout else "2166136261u"
    lines.append(f"unsigned int rolling_hash = {hash_seed};")
    lines.append("void check(bool condition, const char* msg) {")
    lines.append("    total_checks++;")
    lines.append("    rolling_hash = rolling_hash * 16777619u ^ (condition ? 1u : 0u);")
    lines.append("    if (!condition) {")
    lines.append("        std::printf(\"FAILED: %s\\n\", msg);")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    lines.append("int main() {")

    count = 0
    for name, ty, v1, v2, delta, has_comp, has_inc in types:
        for lk in LVALUE_KINDS:
            for op in OPERATIONS:
                count += 1
                case_prefix = f"heldout_{name}_{lk}_{op}" if is_heldout else f"{name}_{lk}_{op}"
                lines.append(f"    // Case {count}: {case_prefix}")
                lines.append("    {")
                
                # Setup lvalue target
                if lk == "local":
                    lines.append(f"        {ty} lv = {v1};")
                    target = "lv"
                elif lk == "global":
                    lines.append(f"        {prefix_pfx}_var_{name} = {v1};")
                    target = f"{prefix_pfx}_var_{name}"
                elif lk == "static_local":
                    lines.append(f"        static {ty} s_var_{name}; s_var_{name} = {v1};")
                    target = f"s_var_{name}"
                elif lk == "struct_member":
                    lines.append(f"        Holder_{prefix_pfx}_{name} h; h.m = {v1};")
                    target = "h.m"
                elif lk == "array_elem":
                    lines.append(f"        {ty} arr[3] = {{ {v1}, {v1}, {v1} }};")
                    target = "arr[1]"
                elif lk == "via_ref":
                    lines.append(f"        {ty} base_val = {v1};")
                    lines.append(f"        {ty}& ref = base_val;")
                    target = "ref"
                elif lk == "via_ptr":
                    lines.append(f"        {ty} base_val = {v1};")
                    lines.append(f"        {ty}* ptr = &base_val;")
                    target = "*ptr"
                elif lk == "via_pptr":
                    lines.append(f"        {ty} base_val = {v1};")
                    lines.append(f"        {ty}* ptr = &base_val;")
                    lines.append(f"        {ty}** pptr = &ptr;")
                    target = "**pptr"

                # Perform operation
                if op == "assign":
                    lines.append(f"        {target} = {v2};")
                    lines.append(f"        check({target} == {v2}, \"{case_prefix}\");")
                elif op == "compound_assign":
                    if has_comp:
                        lines.append(f"        {target} += {delta};")
                        lines.append(f"        check({target} == ({v1}) + {delta}, \"{case_prefix}\");")
                    else:
                        lines.append(f"        {target} = {v2};")
                        lines.append(f"        check({target} == {v2}, \"{case_prefix}\");")
                elif op == "inc_dec":
                    if has_inc:
                        lines.append(f"        ++{target};")
                        lines.append(f"        --{target};")
                        lines.append(f"        check({target} == {v1}, \"{case_prefix}\");")
                    else:
                        lines.append(f"        {target} = {v1};")
                        lines.append(f"        check({target} == {v1}, \"{case_prefix}\");")
                elif op == "read_after_write":
                    lines.append(f"        {target} = {v2};")
                    lines.append(f"        {ty} read_val = {target};")
                    lines.append(f"        check(read_val == {v2}, \"{case_prefix}\");")
                elif op == "pass_by_val":
                    lines.append(f"        take_val_{prefix_pfx}_{name}({target});")
                    lines.append(f"        check({target} == {v1}, \"{case_prefix}\");")
                elif op == "pass_by_ref":
                    lines.append(f"        take_ref_{prefix_pfx}_{name}({target});")
                    lines.append(f"        check({target} == {v1}, \"{case_prefix}\");")
                elif op == "return_by_val":
                    lines.append(f"        {target} = return_{prefix_pfx}_{name}();")
                    lines.append(f"        check({target} == {v2}, \"{case_prefix}\");")

                value_expr = f'({target} != nullptr)' if '*' in ty else f'(int)({target})'
                lines.append(f'        std::printf("CASE {count}: {case_prefix} value=%d\\n", {value_expr});')
                lines.append("    }")
                lines.append("")

    lines.append(f"    std::printf(\"Total test cases executed: %d, verified checks: %d, rolling hash: %u\\n\", {count}, total_checks, rolling_hash);")
    lines.append("    return 0;")
    lines.append("}")
    return "\n".join(lines), count

def generate_file(target_cpp, expected_file, is_heldout):
    cpp_source, count = generate_cpp(is_heldout)
    with open(target_cpp, "w") as f:
        f.write(cpp_source)
    print(f"Generated {count} combinations in {target_cpp}")
    
    bin_gxx = f"/tmp/lval_gxx_{'held' if is_heldout else 'main'}"
    subprocess.run(["g++", "-std=c++17", "-O0", target_cpp, "-o", bin_gxx], check=True)
    out_gxx = subprocess.check_output([bin_gxx], text=True)
    with open(expected_file, "w") as f:
        f.write(out_gxx)
        
    bin_clang = f"/tmp/lval_clang_{'held' if is_heldout else 'main'}"
    subprocess.run(["clang++", "-std=c++17", "-fsanitize=address,undefined", "-O0", target_cpp, "-o", bin_clang], check=True)
    out_clang = subprocess.check_output([bin_clang], text=True)
    assert out_gxx == out_clang, f"g++ and clang++ output differ for {target_cpp}!"
    print(f"GATES PASSED for {target_cpp}: g++ and clang++ ASan/UBSan outputs match!")

def main():
    generate_file("tests/c_basics/lvalue_matrix.cpp", "tests/c_basics/lvalue_matrix.expected", False)
    generate_file("tests/c_basics_heldout/lvalue_matrix.cpp", "tests/c_basics_heldout/lvalue_matrix.expected", True)

if __name__ == "__main__":
    main()
