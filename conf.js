/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        "colors-miscellaneous-sidebar-shadow-drag-over":
          "var(--colors-miscellaneous-sidebar-shadow-drag-over)",
      },
    },
  },
  plugins: [],
};