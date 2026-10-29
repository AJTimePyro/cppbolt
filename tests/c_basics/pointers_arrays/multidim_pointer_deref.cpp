#include <cstdio>

int main() {
    int matrix[2][3] = {{1, 2, 3}, {4, 5, 6}};
    printf("%d\n", *(matrix[1] + 2));
    printf("%d\n", *(*(matrix + 1) + 2));

    auto row_ptr = matrix + 1;
    printf("%d\n", *(*row_ptr + 2));
    printf("%d\n", (*row_ptr)[2]);

    *(*(matrix + 1) + 2) = 42;
    printf("%d\n", matrix[1][2]);

    int tensor[2][2][3] = {
        {{1, 2, 3}, {4, 5, 6}},
        {{7, 8, 9}, {10, 11, 12}}
    };
    printf("%d\n", *(*(*(tensor + 1) + 1) + 2));
    return 0;
}
