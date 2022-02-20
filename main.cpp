#include <iostream>
#include <vector>
#include <complex>
#include <chrono>
#include <SDL2/SDL.h>

const int WIDTH = 1024;
const int HEIGHT = 768;
const int SIZE = WIDTH*HEIGHT;
const double ESC_RADIUS = 1000;

typedef std::complex<long double> Complex;

template<typename T>
struct Vec2D {
    std::vector<T> _data;
    Vec2D() {
        _data.resize(WIDTH*HEIGHT);
    }
    T& operator()(int x, int y) {
        return _data[WIDTH*y+x];
    }
    const T& operator()(int x, int y) const {
        return _data[WIDTH*y+x];
    }

    T& operator[](int i) {
        return _data[i];
    }
    const T& operator[](int i) const {
        return _data[i];
    }
};

struct Rect {
    long double x, y, w, h;
    Rect(
        long double _x=-3,
        long double _y=-1.5,
        long double _w=4,
        long double _h=3
    ) {
        x=_x; y=_y, w=_w, h=_h;
    }
};

struct Zoom: SDL_Rect {
    Zoom() {
        x = 0;
        y = 0;
        w = WIDTH/2;
        h = HEIGHT/2;
    }
};


struct Mandelbrot {
    Rect rect;
    int max_iter;
    Vec2D<Complex> c;
    Vec2D<Complex> z;
    std::vector<bool> diverged;
    std::vector<bool> bounded;
    Vec2D<uint32_t> surface;

    Mandelbrot() {
        max_iter = 30;
        diverged.resize(SIZE);
        bounded.resize(SIZE);
        init(Rect());
    }

    void init(const Rect &_rect) {
        rect = _rect;
        max_iter = 30*std::max(1, int(2-log(rect.w)/log(2)));
        std::cout << rect.x << ' ' << rect.y << ' ' << rect.w << ' ' << rect.h << '\n';

#pragma omp parallel for
        for (int i=0; i<WIDTH; i++)
            for (int j=0; j<HEIGHT; j++)
                c(i,j) = Complex(rect.x + i*rect.w/WIDTH, rect.y+j*rect.h/HEIGHT);

#pragma omp parallel for
        for (int i=0; i<SIZE; i++) {
            z[i] = 0;
            diverged[i] = false;
            bounded[i] = true;
            surface[i] = 0;
        }

    }

    /* Iterate each point up to max_iter
     */
    int steps() {
        int result = 0;
#pragma omp parallel for reduction(+:result)
        for (int i=0; i<SIZE; i++) {
            if (diverged[i])
                continue;
            for (int iter=0; iter<max_iter; iter++) {
                z[i] = z[i]*z[i] + c[i];
                if (bounded[i] && std::norm(z[i]) > ESC_RADIUS) {
                    diverged[i] = true;
                    result += 1;
                    long double fiter = iter + 1 - log(log(std::abs(z[i])))/log(2);
                    surface[i] = 0x10101 * uint32_t(128*(1-fiter/max_iter));
                    break;
                }
            }
        }
        return result;
    }

    /* Check for zooming event and return if a redraw is necessary */
    bool handle_event(SDL_Event &event, const Zoom &zoom) {
        if (event.type == SDL_MOUSEBUTTONDOWN) {
            if (event.button.button == SDL_BUTTON_LEFT) {
                Rect new_rect(
                    rect.x+rect.w/WIDTH*zoom.x,
                    rect.y+rect.h/HEIGHT*zoom.y,
                    rect.w/WIDTH*zoom.w,
                    rect.h/HEIGHT*zoom.h
                );
                init(new_rect);
                return true;
            };
            if (event.button.button == SDL_BUTTON_RIGHT) {
                Rect new_rect(
                    rect.x-rect.w/2,
                    rect.y-rect.h/2,
                    rect.w*2,
                    rect.h*2
                );
                init(new_rect);
                return true;
            };
        }
        return false;
    }
};

struct Timer {
    std::chrono::high_resolution_clock::time_point start;
    Timer() {
        start = std::chrono::high_resolution_clock::now();
    }
    double elapsed() {
        auto now = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(now - start);
        return duration.count();
    }
};


class Framework{
public:
    bool running = true;
    // Contructor which initialize the parameters.
    Framework() {
        SDL_Init(SDL_INIT_VIDEO);       // Initializing SDL as Video
        SDL_CreateWindowAndRenderer(WIDTH, HEIGHT, 0, &window, &renderer);
        SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);      // setting draw color
        SDL_RenderClear(renderer);      // Clear the newly created window
        SDL_RenderPresent(renderer);    // Reflects the changes done in the
                                        //  window.
        texture = SDL_CreateTexture(
            renderer,
            SDL_PIXELFORMAT_ARGB8888,
            SDL_TEXTUREACCESS_STREAMING,
            WIDTH, HEIGHT
        );
        SDL_SetRenderDrawColor(renderer, 255, 0, 0, 255);
    }

    void delay() const {
        SDL_Delay(10);
    }

    void draw(uint32_t* data) {
        SDL_UpdateTexture(texture, NULL, data, 4*WIDTH);
    };

    void flip() {
        SDL_RenderClear(renderer);
        SDL_RenderCopy(renderer, texture, NULL, NULL);
        SDL_RenderDrawRect(renderer, &zoom);
        SDL_RenderPresent(renderer);
    }

    void handle_event(SDL_Event &event) {
        if (event.type == SDL_QUIT) {
            running = false;
        }
        if (event.type == SDL_MOUSEMOTION) {
            zoom.x = std::max(0, std::min(event.motion.x - WIDTH/4, WIDTH/2));
            zoom.y = std::max(0, std::min(event.motion.y - HEIGHT/4, HEIGHT/2));
        }
    }

    const Zoom &current_zoom() const {
        return zoom;
    }

    // Destructor
    ~Framework(){
        SDL_DestroyRenderer(renderer);
        SDL_DestroyWindow(window);
        SDL_Quit();
    }

private:
    SDL_Renderer *renderer = NULL;      // Pointer for the renderer
    SDL_Window *window = NULL;      // Pointer for the window
    SDL_Texture *texture = NULL;
    Zoom zoom;
};

int main() {
    std::cout.precision(20);
    Framework fw;
    Mandelbrot mb;

    SDL_Event event;
    bool redraw = true;
    while (fw.running) {
        if (redraw) {
            mb.steps();
            fw.draw(&mb.surface[0]);
            redraw = false;
        }
        while (SDL_PollEvent(&event)) {
            fw.handle_event(event);
            redraw = mb.handle_event(event, fw.current_zoom());
        }
        fw.flip();
        SDL_Delay(10);
    }
}
