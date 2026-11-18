template <typename T, typename U>
void pair_fn(T t, U u);

template <typename... Ts, typename... Us>
void test_fn(Ts... ts, Us... us) {
    int dummy[] = { (pair_fn(ts, us), 0)... };
    (void)dummy;
}

int main() {
    test_fn(1, 2, 'a');
    return 0;
}
