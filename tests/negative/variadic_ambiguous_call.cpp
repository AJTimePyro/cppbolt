template <typename T, typename... Rest>
void clash(T a, int b, Rest... rest) {}

template <typename T, typename... Rest>
void clash(int a, T b, Rest... rest) {}

int main() {
    clash(1, 2);
    return 0;
}
