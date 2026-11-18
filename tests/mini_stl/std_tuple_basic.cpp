#include <cstdio>
#include <tuple>

int main() {
    auto t1 = std::make_tuple(10, 20, 30);
    printf("%d %d %d\n", std::get<0>(t1), std::get<1>(t1), std::get<2>(t1));

    std::tuple<int, const char*> t2(42, "hello");
    printf("%d %s\n", std::get<0>(t2), std::get<1>(t2));

    return 0;
}
