template <typename T, typename U>
struct Ambiguous {
    int v;
};

template <typename T>
struct Ambiguous<T, int> {
    int v;
};

template <typename U>
struct Ambiguous<int, U> {
    int v;
};

int main() {
    Ambiguous<int, int> a;
    return 0;
}
