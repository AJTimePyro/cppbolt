int main() {
    int x = 10;
    int&& r1 = static_cast<int&&>(x);
    // r1 is a named variable, hence an lvalue.
    // Binding an rvalue reference to an lvalue without std::move must fail.
    int&& r2 = r1;
    return 0;
}
