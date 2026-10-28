#include <cstdio>

// Function pointer comparator referenced ONLY by address

// This function is NEVER called directly by name.
// It is referenced only by address.
bool compare_descending(int a, int b) {
    return a > b;
}

void bubble_sort(int* arr, int n, bool (*cmp)(int, int)) {
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n - 1; ++j) {
            // Indirect call through function pointer!
            if (cmp(arr[j + 1], arr[j])) {
                int tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
        }
    }
}

int main() {
    int data[6];
    data[0] = 5;
    data[1] = 2;
    data[2] = 8;
    data[3] = 1;
    data[4] = 9;
    data[5] = 4;

    // Referenced only by address:
    bubble_sort(data, 6, &compare_descending);

    for (int i = 0; i < 6; ++i) {
        printf("%d ", data[i]);
    }
    printf("\n");
    return 0;
}