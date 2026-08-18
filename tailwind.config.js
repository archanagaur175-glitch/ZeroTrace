/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        base: {
          950: "#0a0e12",
          900: "#0d1117",
          850: "#10161d",
          800: "#161d26",
          700: "#1e2834",
          600: "#2a3746",
        },
        neon: {
          emerald: "#10ffb0",
          cyan: "#22d3ee",
          amber: "#fbbf24",
          red: "#f87171",
        },
      },
      fontFamily: {
        mono: [
          "Cascadia Code",
          "JetBrains Mono",
          "Consolas",
          "ui-monospace",
          "SFMono-Regular",
          "monospace",
        ],
        sans: [
          "Segoe UI",
          "-apple-system",
          "BlinkMacSystemFont",
          "system-ui",
          "sans-serif",
        ],
      },
      boxShadow: {
        glow: "0 0 18px rgba(16, 255, 176, 0.25)",
        "glow-cyan": "0 0 18px rgba(34, 211, 238, 0.22)",
      },
      keyframes: {
        pulseDot: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.35" },
        },
      },
      animation: {
        "pulse-dot": "pulseDot 1.6s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
