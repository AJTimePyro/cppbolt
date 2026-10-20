#include <cstdio>

class ShapeRenderer {
public:
    virtual void render() = 0;
    virtual ~ShapeRenderer() {}
};

class Wireframe : public ShapeRenderer {
public:
    virtual void render() {
        std::printf("Render wireframe mesh\n");
    }
};

class Shaded : public ShapeRenderer {
public:
    virtual void render() {
        std::printf("Render shaded phong model\n");
    }
};

void draw(ShapeRenderer* sr) {
    sr->render();
}

int main() {
    Wireframe w;
    Shaded s;
    draw(&w);
    draw(&s);
    return 0;
}
