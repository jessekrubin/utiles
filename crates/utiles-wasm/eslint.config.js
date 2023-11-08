// eslint.config.js
import jsse from "@jsse/eslint-config";

export default [
  ...jsse({
    typescript: {
      tsconfig: ["./tsconfig.json", "./tsconfig.eslint.json"],
    },
  }),
  {
    files: ["schemas/**/*.schema.json"],
    rules: {
      "unicorn/filename-case": "off",
    },
  },
  {
    files: ["./src/dev/dev.ts", "./src/dev/dev.test.ts", "./src/scratch/**/*"],
    rules: {
      "@typescript-eslint/no-unused-vars": "off",
    },
  },
];
