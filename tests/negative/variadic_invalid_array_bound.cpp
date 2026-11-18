template <int... Ns>
struct A {
    int arr[sizeof...(Ns) / 0];
};

A<1, 2, 3> a;

int main() {
    return 0;
}
