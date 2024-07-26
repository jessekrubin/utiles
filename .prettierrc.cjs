module.exports = {
  plugins: ["prettier-plugin-sql-cst"],
  bracketSpacing: true,
  endOfLine: "lf",
  printWidth: 80,
  semi: true,
  singleQuote: false,
  tabWidth: 2,
  trailingComma: "all",
  useTabs: false,
  overrides: [
    {
      files: ["*.sql"],
      options: { parser: "sqlite" },
    },
  ],
};
