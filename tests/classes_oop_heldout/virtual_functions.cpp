#include <cstdio>

class Widget {
public:
    virtual void render() {
        std::printf("Render Widget generic\n");
    }
    virtual int get_width() {
        return 100;
    }
    virtual ~Widget() {}
};

class Button : public Widget {
    int w;
public:
    Button(int w_) : w(w_) {}
    virtual void render() {
        std::printf("Render Button w=%d\n", w);
    }
    virtual int get_width() {
        return w;
    }
};

class Slider : public Widget {
    int w;
public:
    Slider(int w_) : w(w_) {}
    virtual void render() {
        std::printf("Render Slider w=%d\n", w);
    }
    virtual int get_width() {
        return w;
    }
};

void draw_widget(Widget* w) {
    w->render();
    std::printf("width: %d\n", w->get_width());
}

int main() {
    Button btn(80);
    Slider sld(240);

    draw_widget(&btn);
    draw_widget(&sld);
    return 0;
}
