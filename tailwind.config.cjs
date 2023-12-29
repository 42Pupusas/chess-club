const colors = require('tailwindcss/colors')

/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./public/templates/*.html"],
    plugins: [require("@tailwindcss/forms")],
}

