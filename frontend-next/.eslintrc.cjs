module.exports = {
  reportUnusedDisableDirectives: true,
  plugins: ["@typescript-eslint"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "next",
    "plugin:storybook/recommended",
  ],
  parser: "@typescript-eslint/parser",
};
