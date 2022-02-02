#!/usr/bin/python

from sys import stdout
from dataclasses import dataclass
from math import log
from PIL import Image
import numpy as np
import matplotlib.pyplot as plt

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
esc_radius = 20

c = complex_matrix(xmin, xmax, ymin, ymax, ndata)
z = np.zeros_like(c, dtype=float)
result = np.ones_like(z)
diverged = np.zeros_like(c, dtype=bool)

for iteration in range(max_iter):
    print(iteration)
    z = z ** 2 + c
    diverged_new = (abs(z) > esc_radius)
    np.copyto(
        result,
        (iteration + 1 - np.log(np.log(abs(z)))/log(2)) / max_iter,
        where=diverged_new,
    )
    np.copyto(diverged, True, where=diverged_new)
    np.copyto(z, 2, where=diverged)

plt.imshow(result, 'Greys', extent=(xmin, xmax, ymin, ymax))
plt.show()
