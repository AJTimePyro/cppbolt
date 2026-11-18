template <typename... Ts>
void f(Ts... ts) {}

int main() {
    f<int, double, char>(1, 2);
    return 0;
}
