/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src-ui/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      keyframes: {
        "caret-blink": {
          "0%": { "text-decoration": "none" },
          "100%": { "text-decoration": "underline" },
        }
      },
      animation: {
        "caret-blink": "caret-blink 1s infinite"
      }
    },
  },
  plugins: [],
}

