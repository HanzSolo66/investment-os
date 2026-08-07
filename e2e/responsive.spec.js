const { test, expect } = require("@playwright/test");

async function assertNoHorizontalOverflow(page) {
  const sizes = await page.evaluate(() => ({
    innerWidth: window.innerWidth,
    scrollWidth: document.documentElement.scrollWidth
  }));

  expect(sizes.scrollWidth).toBeLessThanOrEqual(sizes.innerWidth + 1);
}

test("01 - login funciona em viewport mobile", async ({ page }) => {
  await page.goto("/login");

  await expect(page.locator("#username")).toBeVisible();
  await expect(page.locator("#password")).toBeVisible();
  await expect(page.getByRole("button", { name: "Entrar" })).toBeVisible();

  await assertNoHorizontalOverflow(page);
});

test("02 - cadastro funciona em viewport mobile", async ({ page }) => {
  await page.goto("/register");

  await expect(page).toHaveURL(/\/register/);
  await expect(page.locator("#username")).toBeVisible();
  await expect(page.locator("#password")).toBeVisible();
  await expect(page.locator("#password_confirmation")).toBeVisible();
  await expect(page.getByRole("button", { name: "Criar conta" })).toBeVisible();

  await assertNoHorizontalOverflow(page);
});

test("03 - dashboard se adapta a viewport mobile", async ({ page }) => {
  const stamp = `${Date.now()}_${Math.floor(Math.random() * 100000)}`;
  const username = `pw_mobile_${stamp}`;
  const password = `Pw!${stamp}Bb`;

  await page.goto("/register");
  await page.locator("#username").fill(username);
  await page.locator("#password").fill(password);
  await page.locator("#password_confirmation").fill(password);

  await Promise.all([
    page.waitForURL(url => url.pathname === "/", { timeout: 10000 }),
    page.getByRole("button", { name: "Criar conta" }).click()
  ]);

  await expect(
    page.getByRole("heading", { name: "Sua carteira, sob controle." })
  ).toBeVisible();

  await expect(page.getByRole("button", { name: "Adicionar ativo" })).toBeVisible();

  await assertNoHorizontalOverflow(page);
});