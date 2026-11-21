struct NoType {};

template <class T>
int f() {
    char data[sizeof(typename T::type)];
    return (int)sizeof(data);
}

int main() { return f<NoType>(); }
