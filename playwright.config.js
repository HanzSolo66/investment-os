const { defineConfig, devices } = require("@playwright/test");

module.exports = defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 30000,
  expect: { timeout: 7000 },
  reporter: [
    ["list"],
    ["html", { outputFolder: "playwright-report", open: "never" }]
  ],
  use: {
    baseURL: "http://localhost:3000",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure"
  },
  projects: [
    {
      name: "chromium-desktop",
      testIgnore: /responsive\.spec\.js/,
      use: { ...devices["Desktop Chrome"] }
    },
    {
      name: "chromium-mobile",
      testMatch: /responsive\.spec\.js/,
      use: { ...devices["Pixel 5"] }
    }
  ]
});