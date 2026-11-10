// Negative test: negative dependent array bound in template instantiation
template <int N>
struct BadBuffer {
    int data[N];
};

int main() {
    BadBuffer<-5> b;
    return 0;
}
