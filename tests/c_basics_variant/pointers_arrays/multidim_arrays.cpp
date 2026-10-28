#include <cstdio>

struct Vec2 { double a; double b; };
struct Table { int slots[2][4]; short marker; };

int total(const int* row, int n) {
    int sum = 0;
    int i = 0;
    while (i < n) { sum += row[i]; i++; }
    return sum;
}

int main() {
    int rows[3][4] = {{1, 2, 3, 4}, {5, 6, 7, 8}, {9, 10, 11, 12}};
    int i = 0;
    while (i < 3) {
        int j = 0;
        while (j < 4) {
            printf("%d,", rows[i][j]);
            j++;
        }
        i++;
    }
    printf("\n");

    long grid[2][3] = {{100, 200, 300}, {400, 500, 600}};
    printf("%ld %ld %ld\n", grid[1][2], grid[0][0] + grid[1][1], grid[1][0] / 2);

    Vec2 pairs[2][2] = {{{1.5, 2.5}, {3.5, 4.5}}, {{5.5, 6.5}, {7.5, 8.5}}};
    printf("%.1f %.1f %.1f %.1f\n", pairs[0][0].a, pairs[0][1].b, pairs[1][0].a, pairs[1][1].b);

    Table t;
    t.marker = 9;
    for (int r = 0; r < 2; ++r) {
        for (int c = 0; c < 4; ++c) {
            t.slots[r][c] = r * 10 + c;
        }
    }
    printf("%d %d %d %d\n", t.slots[0][3], t.slots[1][0], t.slots[1][3], t.marker);

    printf("%d %d %d\n", total(rows[0], 4), total(rows[1], 4), total(rows[2], 4));
    printf("%d %d\n", (int)sizeof(rows[0]), (int)sizeof(rows));
    return 0;
}
