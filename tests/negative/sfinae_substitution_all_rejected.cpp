template <int N> int only(char (*p)[N - 100]) { return 1; }

int main() {
    return only<4>(0);
}
