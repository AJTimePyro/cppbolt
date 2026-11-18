// Both candidates fail deduction for different reasons. The diagnostic must
// keep both reasons instead of reporting only the last one.
template <typename T> int f(T, T) { return 1; }
template <typename U> int f(U*, U) { return 2; }

int main() {
    return f(1, 2.0);
}
