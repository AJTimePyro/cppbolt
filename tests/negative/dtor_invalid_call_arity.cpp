void one(int);
template<class T>
struct B {
    T v;
    B(T x) : v(x) {}
    ~B() { one(v, v); }
};
void one(int x) {}
struct D : B<int> { D() : B<int>(463) {} };
int main() { D d; }
