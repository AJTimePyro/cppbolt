template <typename T>
void func(T x, const char* s) {
    (void)x;
    (void)s;
}

int main() {
    // 123 cannot convert to const char*, so template resolution must fail.
    func(1, 123);
    return 0;
}
