typedef int T;

int main() {
    T x = 10;
    T* p = &x;
    p->~T(42);
    return 0;
}
