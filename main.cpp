#include <iostream>
#include <vector>
#include <complex>
#include <chrono>
#include <SDL2/SDL.h>

const int WIDTH = 1024;
const int HEIGHT = 768;
const int SIZE = WIDTH*HEIGHT;
const double ESC_RADIUS = 1000;

typedef std::complex<double> Complex;

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


struct Mandelbrot {
    double x1, x2, y1, y2;
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
        init(-3, 1, -1.5, 1.5);
    }

    void init(double _x1, double _x2, double _y1, double _y2) {
        x1 = _x1;
        x2 = _x2;
        y1 = _y1;
        y2 = _y2;

        double stepx = (x2-x1)/WIDTH;
        double stepy = (y2-y1)/HEIGHT;

#pragma omp parallel for
        for (int i=0; i<WIDTH; i++)
            for (int j=0; j<HEIGHT; j++)
                c(i,j) = Complex(x1 + i*stepx, y1+j*stepy);

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
                    double fiter = iter + 1 - log(log(std::abs(z[i])))/log(2);
                    surface[i] = 0x10101 * uint32_t(128*(1-fiter/max_iter));
                    break;
                }
            }
        }
        return result;
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
        SDL_RenderPresent(renderer);
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
};

int main() {
    Framework fw;
    Mandelbrot mb;

    SDL_Event event;
    bool running = true;
    bool drawn = false;
    while (running) {
        if (!drawn) {
            mb.steps();
            fw.draw(&mb.surface[0]);
            fw.flip();
            drawn = true;
        }
        while (SDL_PollEvent(&event)) {
            if (event.type == SDL_QUIT) {
                running = false;
                break;
            }
        }
        SDL_Delay(50);
    }
}
