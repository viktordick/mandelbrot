#!/usr/bin/python
import time
import threading
from math import log

import numpy as np
import pygame


WIDTH = 1024
HEIGHT = 768
esc_radius = 1000

running = True

np.warnings.filterwarnings('ignore')


class Mandelbrot(threading.Thread):
    def __init__(self, xmin, xmax, ymin, ymax):
        self.rect = (xmin, xmax, ymin, ymax)
        self.max_iter = 30
        self.deviation = 0
        self.iteration = 0
        self.ready = False
        self.c = np.zeros((WIDTH, HEIGHT), dtype=np.complex256)
        self.z = np.zeros_like(self.c)
        self.bounded = np.ones_like(self.c, dtype=bool)
        self.diverged = np.zeros_like(self.bounded)
        self.tmp = np.ones_like(self.z, dtype=np.longdouble)
        self.result = np.ones_like(self.z, dtype=np.uint32)
        self.surface = pygame.Surface((WIDTH, HEIGHT), depth=24)
        self.event = threading.Event()
        self.event.set()
        super().__init__()

    def reset(self):
        """
        Reset fields to compute for a newly set rect
        """
        rect = self.rect
        print(int(2-log(rect[1]-rect[0])/log(2)), *rect)
        self.max_iter = 30*max(1, int(2-log(rect[1]-rect[0])/log(2)))
        self.deviation = 0
        re = np.linspace(rect[0], rect[1], WIDTH, dtype=np.longdouble)
        im = np.linspace(rect[2], rect[3], HEIGHT, dtype=np.longdouble)
        self.c = re[:, np.newaxis] + im[np.newaxis, :] * 1j
        np.copyto(self.z, 0)
        np.copyto(self.result, 0)
        np.copyto(self.bounded, True)
        np.copyto(self.diverged, False)
        self.iteration = 0

    def step(self):
        self.z = self.z**2 + self.c
        np.logical_and(np.abs(self.z) > esc_radius, self.bounded,
                       out=self.diverged)
        self.deviation = self.diverged.sum()

        # steps = iteration + 1 - log(log(abs(z)))/log(2)
        # grey = steps/max_iter (0: diverging/white, 1: bounded, black)
        # rgb = 0x10101 * 128*(1-grey)
        np.abs(self.z, out=self.tmp, where=self.diverged)
        np.log(self.tmp, out=self.tmp, where=self.diverged)
        np.log(self.tmp, out=self.tmp, where=self.diverged)
        np.multiply(self.tmp, 128/log(2)/self.max_iter, out=self.tmp,
                    where=self.diverged)
        np.subtract(self.tmp, 128*((self.iteration+1)/self.max_iter-1),
                    out=self.tmp, where=self.diverged)
        np.floor(self.tmp, out=self.tmp, where=self.diverged)
        np.multiply(0x10101, self.tmp, out=self.tmp, where=self.diverged)
        np.copyto(self.result, self.tmp, where=self.diverged, casting='unsafe')

        np.copyto(self.bounded, False, where=self.diverged)
        self.ready = True
        self.iteration += 1

    def finished(self):
        return (
            (self.deviation > 0 and self.deviation < 10) or
            (self.iteration >= self.max_iter)
        )

    def run(self):
        while running:
            if self.event.is_set():
                self.event.clear()
                self.reset()
                starttime = time.monotonic()

            if self.finished():
                if starttime is not None:
                    print(time.monotonic() - starttime)
                    starttime = None
                self.event.wait(timeout=1)
                continue
            self.step()

    def draw(self, screen):
        if self.ready:
            pygame.surfarray.blit_array(self.surface, self.result)
            self.ready = False

        screen.blit(self.surface, (0, 0))

    def timeit(self):
        self.reset()
        t = time.monotonic()
        for i in range(100):
            self.step()
        print(time.monotonic() - t)


class Zoom:
    def __init__(self):
        self.x = 0
        self.y = 0

    def draw(self, screen):
        pygame.draw.lines(
            screen, (255, 0, 0), True, [
                (self.x, self.y),
                (self.x, self.y+HEIGHT/2),
                (self.x+WIDTH/2, self.y+HEIGHT/2),
                (self.x+WIDTH/2, self.y),
            ]
        )

    def update(self, event, rect):
        if event.type in (pygame.MOUSEMOTION, pygame.MOUSEBUTTONDOWN):
            self.x = max(0, min(event.pos[0] - WIDTH/4, WIDTH/2))
            self.y = max(0, min(event.pos[1] - HEIGHT/4, HEIGHT/2))
        if event.type == pygame.MOUSEBUTTONDOWN:
            w = (rect[1]-rect[0])
            h = (rect[3]-rect[2])
            if event.button == 1:
                # zoom in
                rect = (
                    rect[0] + w*self.x/WIDTH,
                    rect[0] + w*self.x/WIDTH + w/2,
                    rect[2] + h*self.y/HEIGHT,
                    rect[2] + h*self.y/HEIGHT + h/2,
                )
            elif event.button == 3:
                # zoom out
                rect = (
                    rect[0] - w/2,
                    rect[1] + w/2,
                    rect[2] - h/2,
                    rect[3] + h/2,
                )
        return rect


pygame.init()
clock = pygame.time.Clock()

mandelbrot = Mandelbrot(-3, 1, -1.5, 1.5)
# mandelbrot.timeit()
# assert False

mandelbrot.start()

zoom = Zoom()

screen = pygame.display.set_mode(
    (WIDTH, HEIGHT),
    pygame.HWSURFACE | pygame.DOUBLEBUF,
)

while running:
    clock.tick(60)
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False
            mandelbrot.event.set()
            break
        rect = zoom.update(event, mandelbrot.rect)
        if rect != mandelbrot.rect:
            mandelbrot.rect = rect
            mandelbrot.event.set()

    mandelbrot.draw(screen)
    zoom.draw(screen)
    pygame.display.update()
