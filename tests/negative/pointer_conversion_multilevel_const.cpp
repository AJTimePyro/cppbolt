int main() {
    int x = 10;
    int* p = &x;
    int** pp = &p;
    const int** q = pp;
    return 0;
}
