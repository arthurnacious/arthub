"""A portrait as text, for a dark page (see scripts/make-portrait.sh).

usage: ascii.py lum.gray sat.gray detail.gray cols rows gamma ramp

The three inputs are 8-bit grayscale rasters at cols x rows: luminance and
saturation pick out the background (bright and unsaturated -> blank), and a
local-contrast version of the photo sets each character's density, lit =
dense, so the face reads as lit against the page.
ramp: classic (ASCII) or block (shade characters)."""
import sys
lum, sat, det = (open(f, 'rb').read() for f in sys.argv[1:4])
C, R, gamma = int(sys.argv[4]), int(sys.argv[5]), float(sys.argv[6])
RAMPS = {'classic': " .,:;-=+*#%@", 'block': " ░▒▓█"}
RAMP = RAMPS[sys.argv[7]]
def bg(i):
    # 0 = subject, 1 = wall; soft edge.
    l, s = lum[i] / 255, sat[i] / 255
    return min(1.0, max(0.0, (l - 0.55) / 0.15)) * min(1.0, max(0.0, (0.28 - s) / 0.12))
vals = [det[i] / 255 for i in range(C * R)]
sub = [v for i, v in enumerate(vals) if bg(i) < 0.5]
sub.sort()
lo, hi = sub[int(len(sub) * 0.03)], sub[int(len(sub) * 0.97)]
rows = []
for y in range(R):
    row = ''
    for x in range(C):
        i = y * C + x
        t = min(1.0, max(0.0, (vals[i] - lo) / (hi - lo))) ** gamma
        t *= 1 - bg(i)
        row += RAMP[min(len(RAMP) - 1, int(t * len(RAMP)))]
    rows.append(row)
print('\n'.join(rows))
