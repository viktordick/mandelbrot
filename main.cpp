#include <iostream>
#include <vector>
#include <complex>
#include <chrono>

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
        return _data[HEIGHT*x+y];
    }
    const T& operator()(int x, int y) const {
        return _data[HEIGHT*x+y];
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
    Vec2D<Complex> c;
    Vec2D<Complex> z;
    std::vector<bool> diverged;
    std::vector<bool> bounded;
    Vec2D<uint32_t> surface;

    Mandelbrot() {
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

    /* Do some step in the iteration, returning how many points newly diverged
     */
    int steps(int iteration) {
        int result = 0;
#pragma omp parallel for reduction(+:result)
        for (int i=0; i<SIZE; i++) {
            if (diverged[i])
                continue;
            for (int iter = 0; iter<100; iter++) {
                z[i] = z[i]*z[i] + c[i];
                if (bounded[i] && std::norm(z[i]) > ESC_RADIUS) {
                    diverged[i] = true;
                    result += 1;
                    double grey = iteration + 1 - log(log(std::abs(z[i])))/log(2);
                    surface[i] = 0x10101 * uint32_t(128*grey);
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

int main() {
    Mandelbrot mb;
    Timer t;
    for (int i=0; i<50; i++) {
        mb.steps(i);
        //std::cout << mb.step(i) << '\n';
    }
    std::cout << t.elapsed() << "ms\n";
    std::cout << "Done\n";
}
