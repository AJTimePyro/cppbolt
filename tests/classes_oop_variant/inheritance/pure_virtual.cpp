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

void draw_pair(ShapeRenderer* first, ShapeRenderer* second) {
    draw(first);
    draw(second);
}

int main() {
    Wireframe w;
    Shaded s;
    draw_pair(&w, &s);
    return 0;
}
