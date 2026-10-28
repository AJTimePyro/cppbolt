struct Point {
    int x;
    Point(int v) : x(v) {}
    Point() : Point(5), x(10) {}
};

int main() {
    Point p;
    return 0;
}
