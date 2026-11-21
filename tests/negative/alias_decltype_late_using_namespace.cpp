namespace ns { int f(); }
using A = decltype(f());
using namespace ns;
int main() {}
