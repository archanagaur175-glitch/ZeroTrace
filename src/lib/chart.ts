export interface ChartPoint {
  t: number;
  up: number;
  down: number;
}

export class RingBuffer {
  private buf: ChartPoint[];
  private head = 0;
  private count = 0;
  readonly capacity: number;

  constructor(capacity: number) {
    this.capacity = capacity;
    this.buf = new Array<ChartPoint>(capacity);
  }

  push(p: ChartPoint): void {
    this.buf[this.head] = p;
    this.head = (this.head + 1) % this.capacity;
    if (this.count < this.capacity) this.count += 1;
  }

  snapshot(): ChartPoint[] {
    const out: ChartPoint[] = new Array(this.count);
    const start = (this.head - this.count + this.capacity) % this.capacity;
    for (let i = 0; i < this.count; i++) {
      out[i] = this.buf[(start + i) % this.capacity];
    }
    return out;
  }
}

export interface DrawOpts {
  up: string;
  down: string;
  grid: string;
  text: string;
  max: number | null;
}

export function drawBandwidth(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  points: ChartPoint[],
  opts: DrawOpts,
  dpr: number,
): void {
  ctx.save();
  ctx.scale(dpr, dpr);
  ctx.clearRect(0, 0, width, height);

  const padTop = 6;
  const padBottom = 16;
  const plotH = height - padTop - padBottom;
  if (plotH <= 0) {
    ctx.restore();
    return;
  }

  ctx.strokeStyle = opts.grid;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let i = 0; i <= 4; i++) {
    const y = padTop + (plotH * i) / 4;
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
  }
  ctx.stroke();

  if (points.length < 2) {
    ctx.fillStyle = opts.text;
    ctx.font = "11px 'Cascadia Code', monospace";
    ctx.fillText("collecting…", 8, padTop + 12);
    ctx.restore();
    return;
  }

  let maxVal = opts.max ?? 0;
  if (opts.max === null) {
    for (const p of points) maxVal = Math.max(maxVal, p.up, p.down);
    if (maxVal <= 0) maxVal = 1;
  }

  const n = points.length;
  const xAt = (i: number): number => (i / (n - 1)) * (width - 4) + 2;

  const drawLine = (pick: (p: ChartPoint) => number, color: string): void => {
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.4;
    ctx.beginPath();
    for (let i = 0; i < n; i++) {
      const y = padTop + plotH - (pick(points[i]) / maxVal) * plotH;
      const x = xAt(i);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
  };

  drawLine((p) => p.down, opts.down);
  drawLine((p) => p.up, opts.up);

  ctx.fillStyle = opts.text;
  ctx.font = "11px 'Cascadia Code', monospace";
  ctx.textAlign = "left";
  ctx.fillText("up", 4, padTop + plotH + 13);
  ctx.fillText("down", width - 40, padTop + plotH + 13);

  ctx.restore();
}
