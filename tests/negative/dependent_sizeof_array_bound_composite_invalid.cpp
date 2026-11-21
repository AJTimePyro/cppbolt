struct NoType {};

template <class T>
struct Invalid {
    char data[sizeof(typename T::type) + 1];
};

Invalid<NoType> invalid;

int main() { return 0; }
