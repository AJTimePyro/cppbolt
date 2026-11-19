static_assert(later == 5, "later is declared below");
constexpr int later = 6;
int main() { return 0; }
