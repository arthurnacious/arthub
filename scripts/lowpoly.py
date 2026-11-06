"""Low-poly a square RGB image: importance-sampled points (dense where the
image has edges), Delaunay triangulation, each triangle filled flat with its
mean colour. Raw RGB in, raw RGB out (ImageMagick does the file formats).

usage: lowpoly.py in.rgb out.rgb size max_points seed bg_spacing feature_spacing [facet_jitter]
(spacings are fractions of the image size; see scripts/make-portrait.sh)
"""
import math, random, sys

src, dst, S, N, seed = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
DB, DF = float(sys.argv[6]), float(sys.argv[7])
JIT = float(sys.argv[8]) if len(sys.argv) > 8 else 0.0
random.seed(seed)
data = open(src, 'rb').read()
px = lambda x, y: data[(y * S + x) * 3:(y * S + x) * 3 + 3]

# Edge strength on a coarse grid (luma Sobel), for importance sampling.
G = 4
W = S // G
luma = [[0.0] * W for _ in range(W)]
for gy in range(W):
    for gx in range(W):
        r, g, b = px(gx * G, gy * G)
        luma[gy][gx] = 0.299 * r + 0.587 * g + 0.114 * b
grad = [[0.0] * W for _ in range(W)]
for y in range(1, W - 1):
    for x in range(1, W - 1):
        L = luma
        gx = (L[y-1][x+1] + 2*L[y][x+1] + L[y+1][x+1]) - (L[y-1][x-1] + 2*L[y][x-1] + L[y+1][x-1])
        gy = (L[y+1][x-1] + 2*L[y+1][x] + L[y+1][x+1]) - (L[y-1][x-1] + 2*L[y-1][x] + L[y-1][x+1])
        grad[y][x] = math.hypot(gx, gy)
gmax = max(max(r) for r in grad) or 1.0

# Points: border, then rejection-sampled by edge strength with a minimum
# spacing that shrinks where detail is high.
pts = [(0, 0), (S - 1, 0), (0, S - 1), (S - 1, S - 1)]
step = S // 10
for i in range(1, 10):
    t = i * step
    pts += [(t, 0), (t, S - 1), (0, t), (S - 1, t)]
cell = 12
buckets = {}
def near(x, y, d):
    cx, cy = int(x // cell), int(y // cell)
    r = int(d // cell) + 1
    for by in range(cy - r, cy + r + 1):
        for bx in range(cx - r, cx + r + 1):
            for (qx, qy) in buckets.get((bx, by), ()):
                if (qx - x) ** 2 + (qy - y) ** 2 < d * d:
                    return True
    return False
for p in pts:
    buckets.setdefault((int(p[0] // cell), int(p[1] // cell)), []).append(p)
tries = 0
while len(pts) < N and tries < N * 200:
    tries += 1
    x, y = random.uniform(1, S - 2), random.uniform(1, S - 2)
    e = (grad[min(int(y) // G, W - 1)][min(int(x) // G, W - 1)] / gmax) ** 0.6
    if random.random() > 0.08 + 0.92 * e:
        continue
    d = S * (DB - (DB - DF) * e)   # sparse on the soft background, tight on features
    if near(x, y, d):
        continue
    pts.append((x, y))
    buckets.setdefault((int(x // cell), int(y // cell)), []).append((x, y))

# Bowyer-Watson Delaunay.
def circum(a, b, c):
    ax, ay = a; bx, by = b; cx, cy = c
    d = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    if abs(d) < 1e-12:
        return (0, 0, float('inf'))
    ux = ((ax*ax + ay*ay) * (by - cy) + (bx*bx + by*by) * (cy - ay) + (cx*cx + cy*cy) * (ay - by)) / d
    uy = ((ax*ax + ay*ay) * (cx - bx) + (bx*bx + by*by) * (ax - cx) + (cx*cx + cy*cy) * (bx - ax)) / d
    return (ux, uy, (ux - ax) ** 2 + (uy - ay) ** 2)
M = S * 10
P = list(pts) + [(-M, -M), (3 * M, -M), (-M, 3 * M)]
n = len(pts)
tris = {(n, n + 1, n + 2): circum(P[n], P[n + 1], P[n + 2])}
for i in range(n):
    x, y = P[i]
    bad = [t for t, (ux, uy, r2) in tris.items() if (x - ux) ** 2 + (y - uy) ** 2 < r2]
    edges = {}
    for t in bad:
        for e in ((t[0], t[1]), (t[1], t[2]), (t[2], t[0])):
            k = tuple(sorted(e))
            edges[k] = edges.get(k, 0) + 1
        del tris[t]
    for (a, b), c in edges.items():
        if c == 1:
            t = (a, b, i)
            tris[t] = circum(P[a], P[b], P[i])
tris = [t for t in tris if max(t) < n]

# Fill each triangle with its mean colour.
out = bytearray(S * S * 3)
for t in tris:
    (ax, ay), (bx, by), (cx, cy) = P[t[0]], P[t[1]], P[t[2]]
    x0, x1 = max(0, int(min(ax, bx, cx))), min(S - 1, int(max(ax, bx, cx)) + 1)
    y0, y1 = max(0, int(min(ay, by, cy))), min(S - 1, int(max(ay, by, cy)) + 1)
    den = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy)
    if abs(den) < 1e-9:
        continue
    inside = []
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            l1 = ((by - cy) * (x + 0.5 - cx) + (cx - bx) * (y + 0.5 - cy)) / den
            l2 = ((cy - ay) * (x + 0.5 - cx) + (ax - cx) * (y + 0.5 - cy)) / den
            if l1 >= -1e-6 and l2 >= -1e-6 and 1 - l1 - l2 >= -1e-6:
                inside.append((x, y))
    if not inside:
        continue
    acc = [0, 0, 0]
    for (x, y) in inside:
        r, g, b = px(x, y)
        acc[0] += r; acc[1] += g; acc[2] += b
    # A little per-facet light so the triangles read from across a room.
    shade = 1.0 + random.uniform(-JIT, JIT)
    c = bytes(min(255, int(v / len(inside) * shade)) for v in acc)
    for (x, y) in inside:
        o = (y * S + x) * 3
        out[o:o + 3] = c
open(dst, 'wb').write(out)
print(f"{len(pts)} points, {len(tris)} triangles")
