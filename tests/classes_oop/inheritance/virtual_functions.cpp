#include <cstdio>

class Shape {
public:
    virtual void draw() {
        std::printf("Draw Shape\n");
    }
    virtual int area() {
        return 0;
    }
};

class Rectangle : public Shape {
    int w, h;
public:
    Rectangle(int w_, int h_) : w(w_), h(h_) {}
    virtual void draw() {
        std::printf("Draw Rectangle %d x %d\n", w, h);
    }
    virtual int area() {
        return w * h;
    }
};

class Circle : public Shape {
    int r;
public:
    Circle(int r_) : r(r_) {}
    virtual void draw() {
        std::printf("Draw Circle r=%d\n", r);
    }
    virtual int area() {
        return 3 * r * r;
    }
};

void render(Shape* s) {
    s->draw();
    std::printf("area: %d\n", s->area());
}

int main() {
    Rectangle rect(4, 5);
    Circle circ(3);
    render(&rect);
    render(&circ);
    return 0;
}
