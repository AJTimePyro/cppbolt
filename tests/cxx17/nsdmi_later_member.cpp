#include <cstdio>

struct S {
  int x = f();
  int y = 7;
  int z = y;
  int w = g(3);
  int c = cf();
  int first = f();
  int f() { return 7; }
  int g(int v) { return v; }
  int g(double v) { return (int)v + 1; }
  int cf() const { return 5; }
};

struct NoInit { int x; };

int main() {
  S s;
  S agg{};
  NoInit n{};
  printf("%d %d %d %d %d %d | %d %d\n", s.x, s.y, s.z, s.w, s.c, s.first,
         agg.x, n.x);
  return 0;
}
