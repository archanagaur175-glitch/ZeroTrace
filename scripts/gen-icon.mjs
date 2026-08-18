// ZeroTrace icon generator — renders a 1024x1024 PNG source with a dark
// rounded tile, an emerald "Z" monogram, and cyan accents. No third-party
// deps: hand-rolled minimal PNG encoder using zlib (Node built-in).
import { deflateSync } from "node:zlib";
import { writeFileSync } from "node:fs";

const SIZE = 1024;
const px = Buffer.alloc(SIZE * SIZE * 4); // RGBA

// ----- help palette --------------------------------------------------------
const BG = [10, 12, 18]; // near-black slate
const TILE = [24, 28, 40]; // base-800
const EMERALD = [16, 255, 176]; // #10ffb0 safe accent
const CYAN = [34, 211, 238]; // #22d3ee data accent

function blend(dst, src, a) {
  const ia = 1 - a;
  dst[0] = Math.round(src[0] * a + dst[0] * ia);
  dst[1] = Math.round(src[1] * a + dst[1] * ia);
  dst[2] = Math.round(src[2] * a + dst[2] * ia);
}

const cx = SIZE / 2;
const cy = SIZE / 2;
const tileR = SIZE * 0.38;
const inset = SIZE * 0.06;
const tileX0 = cx - tileR;
const tileX1 = cx + tileR;
const tileY0 = cy - tileR;
const tileY1 = cy + tileR;

function inRoundedRect(x, y) {
  if (x < tileX0 || x >= tileX1 || y < tileY0 || y >= tileY1) return false;
  const r = tileR * 0.22;
  const dx = Math.min(x - tileX0, tileX1 - 1 - x);
  const dy = Math.min(y - tileY0, tileY1 - 1 - y);
  return dx <= r || dy <= r;
}

// ----- "Z" monogram as three bars ------------------------------------------
const bar = SIZE * 0.055;
const gx0 = cx - tileR * 0.55;
const gx1 = cx + tileR * 0.55;
const gy1 = tileY1 - tileR * 0.28;
const gy0 = tileY0 + tileR * 0.28;
const midY = cy;

function inZ(x, y) {
  // top bar
  if (y > gy0 - bar / 2 && y < gy0 + bar / 2 && x >= gx0 && x <= gx1) return 0;
  // diagonal band from top-right to bottom-left
  const t = (y - gy0) / (gy1 - gy0);
  const xd = gx0 + t * (gx1 - gx0);
  if (Math.abs(x - xd) <= bar / 2 && y >= gy0 && y <= gy1) return 0;
  // bottom bar
  if (y > gy1 - bar / 2 && y < gy1 + bar / 2 && x >= gx0 && x <= gx1) return 0;
  return -1;
}

function inCyanDot(x, y) {
  const d = Math.hypot(x - cy, y - cy);
  return d < tileR * 0.16;
}

for (let y = 0; y < SIZE; y++) {
  for (let x = 0; x < SIZE; x++) {
    const i = (y * SIZE + x) * 4;
    const out = px.subarray(i, i + 4);
    // background
    out[0] = BG[0];
    out[1] = BG[1];
    out[2] = BG[2];
    out[3] = 255;
    if (!inRoundedRect(x, y)) continue;
    // tile fill (simple flat; antialias via supersampling is skipped — the
    // rounded edges are acceptable for a desktop app icon)
    blend(out, TILE, 1);
    // cyan dot centered
    if (inCyanDot(x, y)) {
      blend(out, CYAN, 1);
      continue;
    }
    // emerald Z
    const z = inZ(x, y);
    if (z === 0) blend(out, EMERALD, 1);
  }
}

// ----- minimal PNG encode --------------------------------------------------
function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const t = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crc]);
}

function encodePNG(width, height, rgba) {
  const sig = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type RGBA
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;
  const raw = Buffer.alloc(height * (1 + width * 4));
  for (let y = 0; y < height; y++) {
    raw[y * (1 + width * 4)] = 0; // filter: none
    rgba.copy(raw, y * (1 + width * 4) + 1, y * width * 4, (y + 1) * width * 4);
  }
  const idat = deflateSync(raw, { level: 9 });
  return Buffer.concat([
    sig,
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const out = encodePNG(SIZE, SIZE, px);
writeFileSync(process.argv[2] || "icon-source.png", out);
console.log(`wrote ${process.argv[2] || "icon-source.png"} (${out.length} bytes)`);