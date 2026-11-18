template<class T>
struct B {
    T v;
    B(T x) : v(x) {}
    ~B() {
        int* p = v;
        (void)p;
    }
};
int main() { B<int> b(466); }
