template <typename T, int N>
struct FixedBuffer {
    T data[N];
};

int main() {
    FixedBuffer<int, int> buf;
    return 0;
}
