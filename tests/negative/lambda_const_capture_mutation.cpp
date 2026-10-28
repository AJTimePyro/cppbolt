int main() {
    int x = 10;
    auto l = [x]() {
        x = 20;
    };
    l();
    return 0;
}
