#include <cstdio>
#include <cstdlib>

unsigned int rng = 123456789;
unsigned int next_rand() {
    rng ^= rng << 13;
    rng ^= rng >> 17;
    rng ^= rng << 5;
    return rng;
}

void qsort_rec(int* arr, int l, int r) {
    if (l >= r) return;
    int pivot = arr[l + (r - l) / 2];
    int i = l; int j = r;
    while (i <= j) {
        while (arr[i] < pivot) i++;
        while (arr[j] > pivot) j--;
        if (i <= j) {
            int tmp = arr[i];
            arr[i] = arr[j];
            arr[j] = tmp;
            i++;
            j--;
        }
    }
    if (l < j) qsort_rec(arr, l, j);
    if (i < r) qsort_rec(arr, i, r);
}

int main() {
    const int N = 1000000;
    int* data = (int*)malloc(N * sizeof(int));
    for (int i = 0; i < N; ++i) {
        data[i] = (int)(next_rand() & 0x7FFFFFFF);
    }
    qsort_rec(data, 0, N - 1);
    bool ok = true;
    for (int i = 0; i < N - 1; ++i) {
        if (data[i] > data[i + 1]) { ok = false; break; }
    }
    printf("Sorted %d ints: %s (first=%d, last=%d)\n", N, ok ? "OK" : "FAIL", data[0], data[N - 1]);
    free(data);
    return 0;
}
