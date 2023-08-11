module.exports = {
  reportUnusedDisableDirectives: true,
  plugins: ["@typescript-eslint"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "next",
  ],
  parser: "@typescript-eslint/parser",
};
