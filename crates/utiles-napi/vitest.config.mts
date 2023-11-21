import { defaultExclude, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // ...
    exclude: [...defaultExclude, "_fixtures/**", "fixtures/**"],
    include: ["tests/**/*.test.ts"],
  },
});
