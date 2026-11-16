template <typename... Args>
auto foo(Args... args) {
    return (... + args);
}

int main() {
    foo();
    return 0;
}
