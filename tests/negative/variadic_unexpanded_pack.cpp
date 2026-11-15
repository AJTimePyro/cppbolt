template <typename... Args>
void f(Args... args);

template <typename... Args>
void g(Args... args) {
    f(args);
}

int main() {
    g(1, 2);
    return 0;
}
