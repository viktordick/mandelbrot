#!/usr/bin/python

from math import log
import time
import numpy as np
import matplotlib.pyplot as plt
import threading


ndata = 1024
max_iter = 50
esc_radius = 20
np.warnings.filterwarnings('ignore')


def complex_matrix(xmin, xmax, ymin, ymax):
    re = np.linspace(xmin, xmax, ndata, dtype=np.longdouble)
    im = np.linspace(ymin, ymax, ndata, dtype=np.longdouble)
    return re[np.newaxis, :] + im[:, np.newaxis] * 1j


class Mandelbrot(threading.Thread):
    def __init__(self, xmin, xmax, ymin, ymax):
        self.draw_version = 0
        self.running = True
        self.rect = (xmin, xmax, ymin, ymax)
        self.data = None
        super().__init__()

    def run(self):
        rect = None
        deviation = 100

        while self.running:
            if rect != self.rect:
                rect = self.rect
                print('resize:', rect)
                c = complex_matrix(*rect)
                z = np.zeros_like(c, dtype=np.longdouble)
                result = np.ones_like(z)
                diverged = np.zeros_like(c, dtype=bool)
                iteration = 0
                deviation = 100

            if iteration >= max_iter:
                time.sleep(1)
                continue

            print(self.draw_version, deviation)
            z = z ** 2 + c
            diverged_new = (abs(z) > esc_radius)
            np.copyto(
                result,
                (iteration + 1 - np.log(np.log(abs(z)))/log(2)) / max_iter,
                where=diverged_new,
            )
            np.copyto(diverged, True, where=diverged_new)
            np.copyto(z, 2, where=diverged)
            if self.data is not None:
                deviation = np.linalg.norm(self.data - result)
            self.data = result.copy()
            iteration += 1
            self.draw_version += 1


mandelbrot = Mandelbrot(-2, -0.5, -1.5, 1.5)
mandelbrot.start()

plt.ion()
plt.show()


def on_zoom(ax):
    mandelbrot.rect = ax.get_xlim() + ax.get_ylim()


def on_close(event):
    mandelbrot.running = False


fig, ax = plt.subplots()
c1 = ax.callbacks.connect('xlim_changed', on_zoom)
c2 = ax.callbacks.connect('ylim_changed', on_zoom)
fig.canvas.mpl_connect('close_event', on_close)

drawn = 0
while mandelbrot.running:
    delay = 5
    if mandelbrot.draw_version > drawn:
        delay = 1
        drawn = mandelbrot.draw_version
        print('Draw', drawn)
        plt.imshow(
            mandelbrot.data,
            'Greys',
            extent=mandelbrot.rect,
            origin='lower',
        )
    plt.pause(delay)

mandelbrot.join()

plt.ioff()
plt.show()
