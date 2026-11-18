template <typename... Ts>
struct Outer {
    template <typename... Us>
    struct Inner {
        int x;
    };
};

int main() {
    Outer<int>::NoSuchInner<float> var;
    return 0;
}
