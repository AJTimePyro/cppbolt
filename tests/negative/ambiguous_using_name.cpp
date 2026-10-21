namespace a { int x = 1; }
namespace b { int x = 2; }
using a::x;
using namespace b;
int main() {
    return x;
}
