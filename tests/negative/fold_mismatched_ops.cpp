template <typename... Args>
auto foo(Args... args) {
    return (args + ... - 0);
}
