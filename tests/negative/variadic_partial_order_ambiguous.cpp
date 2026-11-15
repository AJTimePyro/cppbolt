template <typename T, typename U>
void order(T*, U) {}

template <typename T, typename U>
void order(T, U*) {}

int main() {
    int x = 0;
    order(&x, &x);
    return 0;
}
