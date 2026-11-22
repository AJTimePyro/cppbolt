#include <cstdio>
#include <iterator>
#include <algorithm>

struct Item {
    int value;
};

int main() {
    int arr[] = {10, 20, 30, 40};
    int* p = arr;
    std::advance(p, 2);
    std::printf("*p: %d\n", *p);
    std::printf("dist: %ld\n", (long)std::distance(arr, p));
    std::iter_swap(arr, arr + 3);
    std::printf("swap: %d %d\n", arr[0], arr[3]);
    auto next_p = std::next(arr, 1);
    std::printf("next: %d\n", *next_p);
    auto prev_p = std::prev(arr + 3, 1);
    std::printf("prev: %d\n", *prev_p);

    char letters[] = {'a', 'b', 'c'};
    std::reverse_iterator<char*> chars(letters + 3);
    std::printf("reverse char: %c\n", *chars);
    ++chars;
    std::printf("reverse char: %c\n", *chars);
    ++chars;
    std::printf("reverse char: %c\n", *chars);

    Item items[] = {{7}, {8}};
    std::reverse_iterator<Item*> item_it(items + 2);
    std::printf("reverse item: %d\n", item_it->value);
    return 0;
}
