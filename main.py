#!/usr/bin/python

from math import log
import numpy as np
import matplotlib.pyplot as plt
import threading

np.warnings.filterwarnings('ignore')

def complex_matrix(xmin, xmax, ymin, ymax, pixel_density):
    re = np.linspace(xmin, xmax, int((xmax - xmin) * pixel_density))
    im = np.linspace(ymin, ymax, int((ymax - ymin) * pixel_density))
    return re[np.newaxis, :] + im[:, np.newaxis] * 1j

xmin = -2
xmax = 0.5
ymin = -1.5
ymax = 1.5
ndata = 1024
max_iter = 50
esc_radius = 10 

class Mandelbrot(threading.Thread):
    def __init__(self):
        self.iteration = 0
        self.finished = False
        self.data = np.zeros((ndata, ndata))
        super().__init__()

    def run(self):
        c = complex_matrix(xmin, xmax, ymin, ymax, ndata)
        z = np.zeros_like(c, dtype=float)
        result = np.ones_like(z)
        diverged = np.zeros_like(c, dtype=bool)

        while self.iteration < max_iter:
            print(self.iteration)
            z = z ** 2 + c
            diverged_new = (abs(z) > esc_radius)
            np.copyto(
                result,
                (self.iteration + 1 - np.log(np.log(abs(z)))/log(2)) / max_iter,
                where=diverged_new,
            )
            np.copyto(diverged, True, where=diverged_new)
            np.copyto(z, 2, where=diverged)

            self.data = result.copy()
            self.iteration += 1

        self.finished = True

mandelbrot = Mandelbrot()
mandelbrot.start()

plt.ion()
plt.show()
drawn = 0
while True:
    if mandelbrot.iteration > drawn:
        drawn = mandelbrot.iteration
        print('Drawing', drawn)
        plt.imshow(mandelbrot.data, 'Greys', extent=(xmin, xmax, ymin, ymax))
    elif mandelbrot.finished:
        break
    plt.pause(1)

mandelbrot.join()

plt.ioff()
plt.show()
