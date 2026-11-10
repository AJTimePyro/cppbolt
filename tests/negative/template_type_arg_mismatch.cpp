// Negative test: non-type argument passed to type template parameter
template <typename T>
struct Box {};

int main() {
    Box<42> b;
    return 0;
}
