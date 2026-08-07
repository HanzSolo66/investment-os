const { test, expect } = require("@playwright/test");

test.describe.configure({ mode: "serial" });

function monitorPage(page) {
  const errors = [];

  page.on("pageerror", error => {
    errors.push(`pageerror: ${error.message}`);
  });

  page.on("console", message => {
    if (message.type() === "error") {
      errors.push(`console.error: ${message.text()}`);
    }
  });

  page.on("response", response => {
    if (response.status() >= 500) {
      errors.push(
        `HTTP ${response.status()}: ${response.request().method()} ${response.url()}`
      );
    }
  });

  return errors;
}

function credentials(prefix) {
  const stamp = `${Date.now()}_${Math.floor(Math.random() * 100000)}`;
  return {
    username: `${prefix}_${stamp}`,
    password: `Pw!${stamp}Aa`
  };
}

async function register(page, username, password) {
  await page.goto("/register");
  await expect(page).toHaveURL(/\/register/);
  await expect(page.locator("#username")).toBeVisible();
  await expect(page.locator("#password")).toBeVisible();
  await expect(page.locator("#password_confirmation")).toBeVisible();

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
}

async function addAsset(page, name, unitValue, quantity) {
  await page.getByRole("button", { name: "Adicionar ativo" }).click();

  const dialog = page.locator("#asset-modal");
  await expect(dialog).toHaveAttribute("aria-hidden", "false");
  await expect(dialog.locator("#asset-modal-title")).toHaveText("Adicionar ativo");

  await dialog.locator("#name").fill(name);
  await dialog.locator("#unit_value").fill(String(unitValue));
  await dialog.locator("#quantity").fill(String(quantity));

  await Promise.all([
    page.waitForURL(url => url.pathname === "/", { timeout: 10000 }),
    dialog.locator("#submit-asset").click()
  ]);

  const row = page.locator("tbody tr", { hasText: name });
  await expect(row).toBeVisible();

  return row;
}

test("01 - acesso sem sessao redireciona para login", async ({ page }) => {
  const errors = monitorPage(page);

  await page.goto("/");
  await expect(page).toHaveURL(/\/login/);
  await expect(page.locator("#username")).toBeVisible();
  await expect(page.locator("#password")).toBeVisible();

  expect(errors).toEqual([]);
});

test("02 - cadastro, dashboard e CRUD completo", async ({ page }) => {
  const errors = monitorPage(page);
  const { username, password } = credentials("pw_crud");
  const assetName = `Playwright-${Date.now()}`;

  await register(page, username, password);

  let row = await addAsset(page, assetName, 25, 2);

  await expect(row.locator("td").nth(1)).toContainText("2");
  await expect(row).toContainText(assetName);

  await row.getByRole("button", { name: "Editar" }).click();

  const editDialog = page.locator("#asset-modal");
  await expect(editDialog.locator("#asset-modal-title")).toHaveText("Editar ativo");
  await expect(editDialog.locator("#name")).toHaveValue(assetName);

  await editDialog.locator("#quantity").fill("3");

  await Promise.all([
    page.waitForURL(url => url.pathname === "/", { timeout: 10000 }),
    editDialog.locator("#submit-asset").click()
  ]);

  row = page.locator("tbody tr", { hasText: assetName });
  await expect(row).toBeVisible();
  await expect(row.locator("td").nth(1)).toContainText("3");

  await row.getByRole("button", { name: "Excluir" }).click();

  const deleteDialog = page.locator("#delete-modal");
  await expect(deleteDialog).toHaveAttribute("aria-hidden", "false");
  await expect(deleteDialog.locator("#delete-asset-name")).toHaveText(assetName);

  await Promise.all([
    page.waitForURL(url => url.pathname === "/", { timeout: 10000 }),
    deleteDialog.locator("#confirm-delete-button").click()
  ]);

  await expect(page.locator("tbody tr", { hasText: assetName })).toHaveCount(0);

  expect(errors).toEqual([]);
});

test("03 - logout encerra a sessao e protege o dashboard", async ({ page }) => {
  const errors = monitorPage(page);
  const { username, password } = credentials("pw_logout");

  await register(page, username, password);

    const logoutButton = page.locator(".logout-button");
  await expect(logoutButton).toBeVisible();

  await Promise.all([
    page.waitForURL(url => url.pathname === "/login", { timeout: 10000 }),
    logoutButton.click()
  ]);

  await expect(page).toHaveURL(/\/login/);
    await expect(page.locator("#alert.show")).toBeVisible();

  await page.goto("/");
  await expect(page).toHaveURL(/\/login/);

  expect(errors).toEqual([]);
});

test("04 - carteiras ficam isoladas entre usuarios", async ({ browser }) => {
  const a = credentials("pw_iso_a");
  const b = credentials("pw_iso_b");
  const assetName = `Privado-${Date.now()}`;

  const contextA = await browser.newContext();
  const pageA = await contextA.newPage();
  const errorsA = monitorPage(pageA);

  await register(pageA, a.username, a.password);
  await addAsset(pageA, assetName, 10, 1);
  await expect(pageA.locator("tbody tr", { hasText: assetName })).toBeVisible();

  const contextB = await browser.newContext();
  const pageB = await contextB.newPage();
  const errorsB = monitorPage(pageB);

  await register(pageB, b.username, b.password);

  await expect(pageB.getByText(assetName, { exact: true })).toHaveCount(0);

  const apiResponse = await pageB.request.get("/api/assets");
  expect(apiResponse.ok()).toBeTruthy();

  const assetsB = await apiResponse.json();
  expect(JSON.stringify(assetsB)).not.toContain(assetName);

  await pageA.reload();
  await expect(pageA.locator("tbody tr", { hasText: assetName })).toBeVisible();

  expect(errorsA).toEqual([]);
  expect(errorsB).toEqual([]);

  await contextA.close();
  await contextB.close();
});