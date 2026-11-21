struct HasPrivate {
private:
    typedef int type;
};

template<class T> int g() { return sizeof(typename T::type); }
int x = g<HasPrivate>();

int main() { return 0; }
