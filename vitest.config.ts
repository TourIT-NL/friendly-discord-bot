import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: true,
    environment: "jsdom", // or 'node' if testing Node.js specific code
    setupFiles: "./src/setupTests.ts", // Example setup file for React Testing Library
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"], // Output formats for coverage reports
      reportsDirectory: "./coverage/frontend", // Directory for coverage reports
      exclude: ["node_modules/", "src/main.tsx", "src/App.tsx"], // Exclude files from coverage
    },
  },
});
