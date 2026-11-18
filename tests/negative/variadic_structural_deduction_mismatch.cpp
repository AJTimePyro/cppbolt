template <typename... Ts>
struct Tuple {};

template <typename First, typename... Rest>
void g(Tuple<First, Rest...>);

int main() {
    g(Tuple<>{});
    return 0;
}
