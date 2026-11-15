void foo(int);

template <typename... Args>
void bar(Args... args) {
    foo(args)...;
}

int main() {
    bar(1, 2);
    return 0;
}
