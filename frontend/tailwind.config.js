/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        aquamarine: {
          50: "#e6fff7", 100: "#cdfeef", 200: "#9afede", 300: "#68fdce",
          400: "#35fdbe", 500: "#03fcad", 600: "#02ca8b", 700: "#029768",
          800: "#016545", 900: "#013223", 950: "#002318",
        },
        "maya-blue": {
          50: "#e6f4ff", 100: "#cde9fe", 200: "#9ad3fe", 300: "#68bcfd",
          400: "#35a6fd", 500: "#0390fc", 600: "#0273ca", 700: "#025697",
          800: "#013a65", 900: "#011d32", 950: "#001423",
        },
        "alice-blue": {
          50: "#e5f4ff", 100: "#cce8ff", 200: "#99d1ff", 300: "#66baff",
          400: "#33a3ff", 500: "#008cff", 600: "#0070cc", 700: "#005499",
          800: "#003866", 900: "#001c33", 950: "#001424",
        },
        "lavender-grey": {
          50: "#eeeff6", 100: "#dedeed", 200: "#bcbddc", 300: "#9b9cca",
          400: "#797bb9", 500: "#585ba7", 600: "#464886", 700: "#353664",
          800: "#232443", 900: "#121221", 950: "#0c0d17",
        },
        black: {
          50: "#f1f2f4", 100: "#e3e6e8", 200: "#c6ccd2", 300: "#aab3bb",
          400: "#8e99a4", 500: "#717f8e", 600: "#5b6671", 700: "#444c55",
          800: "#2d3339", 900: "#17191c", 950: "#101214",
        },
      },
      fontFamily: {
        display: ["Amiri", "Georgia", "Times New Roman", "serif"],
        body: ["Segoe UI", "Tahoma", "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [],
};