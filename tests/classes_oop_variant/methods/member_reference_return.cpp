#include <cstdio>

class Store {
public:
    int cells[2];
    Store() {
        cells[0] = 7;
        cells[1] = 8;
    }
    int& element(int i) { return cells[i]; }
};

int main() {
    Store store;
    store.element(0) = 90;
    store.element(1) = 91;
    int& pick = store.element(1);
    std::printf("%d %d %d\n", store.element(0), store.element(1), pick);
    return 0;
}
