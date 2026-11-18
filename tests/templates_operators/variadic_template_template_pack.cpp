#include <cstdio>

template <template <typename...> class... Containers>
struct ContainerHolder {
    static int count() {
        return sizeof...(Containers);
    }
};

template <template <typename...> class First, template <typename...> class... Rest>
struct HeadContainer {
    static int rest_count() {
        return sizeof...(Rest);
    }
};

template <typename T>
struct MyVec {};

template <typename T>
struct MyList {};

template <typename T>
struct MyDeque {};

int main() {
    std::printf("%d\n", ContainerHolder<MyVec, MyList, MyDeque>::count());
    std::printf("%d\n", ContainerHolder<>::count());
    std::printf("%d\n", HeadContainer<MyVec, MyList, MyDeque>::rest_count());
    return 0;
}
