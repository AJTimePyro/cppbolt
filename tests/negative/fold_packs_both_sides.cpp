template <typename... Args>
auto foo(Args... args) {
    return (args + ... + args);
}

int main() {
    foo(1, 2);
    return 0;
}
