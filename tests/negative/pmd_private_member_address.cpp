class C {
  int priv;
public:
  int pub;
};
int main() {
  auto p = &C::priv;   // private member address taken outside the class
  (void)p;
  return 0;
}
