module.exports = {
  trailingComma: "es5",
  tabWidth: 2,
  semi: true,
  singleQuote: false,
  plugins: [
    "@trivago/prettier-plugin-sort-imports",
    require("prettier-plugin-tailwindcss"),
  ],
  importOrder: ["^[./]"],
  importOrderSeparation: true,
}
