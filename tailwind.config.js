/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {}
  },
  plugins: [require('@tailwindcss/container-queries'), require('@tailwindcss/forms')],
}
