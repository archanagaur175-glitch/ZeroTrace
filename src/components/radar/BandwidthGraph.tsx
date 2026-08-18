import { useEffect, useRef } from "react";
import { drawBandwidth, type ChartPoint } from "../../lib/chart";

export function BandwidthGraph({
  points,
  height,
}: {
  points: ChartPoint[];
  height: number;
}) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = Math.max(1, Math.floor(rect.width * dpr));
    canvas.height = Math.floor(height * dpr);
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    drawBandwidth(
      ctx,
      rect.width,
      height,
      points,
      {
        up: "#10ffb0",
        down: "#22d3ee",
        grid: "rgba(42,55,70,0.55)",
        text: "#8b98a9",
        max: null,
      },
      dpr,
    );
  });

  return <canvas ref={ref} style={{ width: "100%", height }} />;
}
