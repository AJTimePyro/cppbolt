struct NoTypeBase {};

template <class T>
struct BadWrapper : NoTypeBase {};

template <class T>
struct BadHolder {
    char data[sizeof(typename BadWrapper<T>::type)];
};

BadHolder<int> bad;

int main() { return 0; }
