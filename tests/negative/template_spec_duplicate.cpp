template <typename T>
struct DupSpec {
    int x;
};

template <>
struct DupSpec<int> {
    int x;
};

template <>
struct DupSpec<int> {
    int x;
};

int main() {
    DupSpec<int> d;
    return 0;
}
