template <typename... Args>
auto foo(Args... args) {
    return (1 + ... + 2);
}

int main() {
    foo(1, 2);
    return 0;
}
