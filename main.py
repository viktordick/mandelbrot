#!/usr/bin/python
import time
import threading
from math import log

import numpy as np
import pygame


WIDTH = 1024
HEIGHT = 768
esc_radius = 100

running = True

np.warnings.filterwarnings('ignore')


class Mandelbrot(threading.Thread):
    def __init__(self, xmin, xmax, ymin, ymax):
        self.rect = (xmin, xmax, ymin, ymax)
        self.data = None
        self.surface = pygame.Surface((WIDTH, HEIGHT), depth=24)
        super().__init__()

    def run(self):
        rect = None

        while running:
            if rect != self.rect:
                rect = self.rect
                print(int(2-log(rect[1]-rect[0])/log(2)), *rect)
                max_iter = 30*max(1, int(2-log(rect[1]-rect[0])/log(2)))
                iteration = 0
                deviation = 0
                starttime = time.monotonic()
                self.data = None
                re = np.linspace(rect[0], rect[1], WIDTH, dtype=np.longdouble)
                im = np.linspace(rect[2], rect[3], HEIGHT, dtype=np.longdouble)
                c = re[:, np.newaxis] + im[np.newaxis, :] * 1j
                z = np.zeros_like(c, dtype=np.complex256)
                result = np.ones_like(z)
                diverged = np.zeros_like(c, dtype=bool)
                diverged_new = np.zeros_like(c, dtype=bool)

            if (deviation > 0 and deviation < 10) or iteration >= max_iter:
                if starttime is not None:
                    print(time.monotonic() - starttime)
                    starttime = None
                time.sleep(1)
                continue

            np.copyto(z, z**2+c)
            np.logical_and(
                abs(z) > esc_radius,
                np.logical_not(diverged),
                out=diverged_new,
            )
            deviation = diverged_new.sum()
            np.copyto(
                result,
                (iteration + 1 - np.log(np.log(abs(z)))/log(2)) / max_iter,
                where=diverged_new,
            )
            np.copyto(diverged, True, where=diverged_new)
            # Convert to greyscale
            self.data = (257*256+1) * np.array(128*(1-result), dtype=np.uint32)
            iteration += 1

    def draw(self, screen):
        if self.data is not None:
            pygame.surfarray.blit_array(self.surface, self.data)
            self.data = None

        screen.blit(self.surface, (0, 0))


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
            break
        mandelbrot.rect = zoom.update(event, mandelbrot.rect)

    mandelbrot.draw(screen)
    zoom.draw(screen)
    pygame.display.update()
