namespace N1 { int a = 1; }
namespace N2 { int a = 2; }
int main() {
    namespace Local = N1;
    namespace Local = N2;
    return 0;
}
