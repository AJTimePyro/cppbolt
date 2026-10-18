// Held-out variant: Function pointer comparator referenced ONLY by address
bool compare_ascending(int a, int b) {
    return a < b;
}

void selection_sort(int* arr, int n, bool (*cmp)(int, int)) {
    for (int i = 0; i < n - 1; ++i) {
        int best = i;
        for (int j = i + 1; j < n; ++j) {
            if (cmp(arr[j], arr[best])) {
                best = j;
            }
        }
        int tmp = arr[i];
        arr[i] = arr[best];
        arr[best] = tmp;
    }
}

int main() {
    int data[5];
    data[0] = 73;
    data[1] = 12;
    data[2] = 99;
    data[3] = 4;
    data[4] = 42;

    selection_sort(data, 5, &compare_ascending);

    for (int i = 0; i < 5; ++i) {
        printf("%d ", data[i]);
    }
    printf("\n");
    return 0;
}
