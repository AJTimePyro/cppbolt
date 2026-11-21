template <int N> struct Box { static const int v = N; };
constexpr int early = 4;
int valid = Box<early>::v;
int invalid = Box<late>::v;
constexpr int late = 9;
int main() { return 0; }
