// Deterministic per-project "fingerprint" pattern, shown on a project
// card/slider when it has no uploaded images. Same seed (a project id)
// always produces the same art, so it doesn't flicker between renders,
// but different projects look visibly distinct from each other.
const PALETTE = ['·', '░', '▒', '▓', '█']

function hashSeed(seed: string): number {
  let hash = 0
  for (let i = 0; i < seed.length; i++) {
    hash = (Math.imul(hash, 31) + seed.charCodeAt(i)) | 0
  }
  return hash >>> 0
}

// mulberry32 - small, fast, seedable PRNG. Not cryptographic; just needs
// to be deterministic and look reasonably non-repetitive at this scale.
function mulberry32(seed: number): () => number {
  let a = seed

  return function () {
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

export function generateAsciiArt(seed: string, cols = 16, rows = 7): string {
  const random = mulberry32(hashSeed(seed))
  const lines: string[] = []

  for (let row = 0; row < rows; row++) {
    let line = ''
    for (let col = 0; col < cols; col++) {
      // Biased toward blank so it reads as a sparse pattern rather than
      // solid noise.
      line += random() > 0.55 ? PALETTE[Math.floor(random() * PALETTE.length)] : ' '
    }
    lines.push(line)
  }

  return lines.join('\n')
}
