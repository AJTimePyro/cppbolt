#include <cstdio>

struct Point { int x; int y; };
struct Grid { int cells[2][3]; char tag; };

int sum_row(int* row, int n) {
    int total = 0;
    for (int i = 0; i < n; ++i) { total = total + row[i]; }
    return total;
}

int main() {
    int m[2][3] = {{0, 1, 2}, {10, 11, 12}};
    for (int i = 0; i < 2; ++i) {
        for (int j = 0; j < 3; ++j) { printf("%d ", m[i][j]); }
    }
    printf("\n");

    int cube[2][2][2] = {{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}};
    for (int i = 0; i < 2; ++i)
        for (int j = 0; j < 2; ++j)
            for (int k = 0; k < 2; ++k) printf("%d ", cube[i][j][k]);
    printf("\n");

    char letters[2][3] = {{'a', 'b', 'c'}, {'d', 'e', 'f'}};
    short shorts[2][2] = {{1, 2}, {3, 4}};
    double doubles[2][2] = {{1.5, 2.5}, {3.5, 4.5}};
    printf("%c%c %d %d %.1f %.1f\n", letters[0][2], letters[1][0], shorts[1][1], shorts[0][0], doubles[0][1], doubles[1][0]);

    Point pts[2][2] = {{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}};
    printf("%d %d %d %d\n", pts[0][1].x, pts[1][0].y, pts[1][1].x, pts[0][0].y);

    Grid g;
    g.cells[0][0] = 7; g.cells[0][1] = 8; g.cells[0][2] = 9;
    g.cells[1][0] = 10; g.cells[1][1] = 11; g.cells[1][2] = 12;
    g.tag = 'G';
    printf("%d %d %d %d %d %d %c\n", g.cells[0][0], g.cells[0][1], g.cells[0][2], g.cells[1][0], g.cells[1][1], g.cells[1][2], g.tag);

    printf("%d %d %d\n", sum_row(m[0], 3), sum_row(m[1], 3), (int)sizeof(m[0]));
    printf("%d %d\n", (int)sizeof(cube[0]), (int)sizeof(cube[0][0]));
    int* flat = m[0];
    printf("%d %d %d\n", flat[1], flat[4], flat[5]);
    return 0;
}
