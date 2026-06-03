/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        "sidebar-bg": "#1b1b1b",
        "main-bg": "#0d0d0d",
        "astra-red": "#f64a46",
        "astra-red-50": "rgba(246,74,70,0.5)",
        "astra-red-21": "rgba(246,74,70,0.21)",
        "online-green": "#15ff00",
        "online-green-bg": "rgba(21,255,0,0.18)",
        "text-dim": "rgba(255,255,255,0.3)",
        "colors-miscellaneous-sidebar-shadow-drag-over":
          "var(--colors-miscellaneous-sidebar-shadow-drag-over)",
      },
      fontFamily: {
        armor: ["'Armor Piercing 2.0 BB-Regular'", "Helvetica", "sans-serif"],
        proxima: ["'Proxima Nova-Bold'", "Helvetica", "sans-serif"],
        "proxima-semi": ["'Proxima Nova-Semibold'", "Helvetica", "sans-serif"],
      },
      width: {
        "sidebar": "157px",
      },
      height: {
        "launcher": "640px",
      },
    },
  },
  plugins: [],
};
