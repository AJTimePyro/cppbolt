#include <cstdio>

struct Table {
    int cells[3];
    Table() { cells[0] = 1; cells[1] = 2; cells[2] = 3; }
    int& operator[](int i) {
        printf("edit%d\n", i);
        return cells[i];
    }
    const int& operator[](int i) const {
        printf("read%d\n", i);
        return cells[i];
    }
};

int main() {
    Table t;
    const Table& view = t;
    const Table* p = &t;
    printf("%d\n", t[1]);
    printf("%d\n", view[0]);
    printf("%d\n", (*p)[2]);
    t[0] = 9;
    printf("%d\n", view[0]);
    printf("%d\n", view[1]);
    printf("%d\n", view[2]);
    const int& first = view[0];
    printf("%d\n", first);
    return 0;
}
