#include <iostream>

template <typename T>
struct Inspector {
    using Tag = const char*;
};

template <typename T>
struct Inspector<T*> {
    using Tag = long long;
    struct Item {
        int code;
        int count;
    };
};

int main() {
    Inspector<int*>::Tag tag = 100LL;
    Inspector<int*>::Item item{7, 14};
    std::cout << tag << " " << item.code << " " << item.count << std::endl;
    return 0;
}
