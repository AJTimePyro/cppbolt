template <typename T>
struct Holder {
    T value;
};

template <>
struct Holder<int> {
    int value;
};

int main() {
    return 0;
}
