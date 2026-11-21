struct NoType {};

template <class T>
struct S {
    static const int sc = sizeof(typename T::type);
};

S<NoType> s;

int main() { return 0; }
