struct Base {
private:
  typedef int secret;
};
struct Derived : Base {};
using X = Derived::secret;
int main() { return 0; }
