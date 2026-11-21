struct NoType {};
template <class T>
struct Holder {
    char data[sizeof(typename T::type)];
};
Holder<NoType> h;
int main() { return 0; }
