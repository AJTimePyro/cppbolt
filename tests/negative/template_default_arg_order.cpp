// Negative test: template parameter without default follows parameter with default
template <typename T = int, typename U>
struct InvalidOrder {};

int main() {
    return 0;
}
