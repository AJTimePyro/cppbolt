template <typename T>
struct Recurse {
    Recurse<Recurse<T>> next;
};

int main() {
    Recurse<int> r;
    return 0;
}
