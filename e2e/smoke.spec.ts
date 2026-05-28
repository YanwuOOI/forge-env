import { test, expect } from '@playwright/test';

test.describe('Forge Env — browser preview smoke tests', () => {
  test('loads the app shell', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('text=Forge Env')).toBeVisible();
    await expect(page.locator('text=Soft industrial control room')).toBeVisible();
  });

  test('displays overview section by default', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible();
    await expect(page.locator('text=Workspace pulse')).toBeVisible();
  });

  test('navigates to hosts section', async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Hosts")');
    await expect(page.locator('text=Machine topology')).toBeVisible();
    await expect(page.locator('text=Browser Preview Host')).toBeVisible();
  });

  test('navigates to languages section', async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Languages")');
    await expect(page.locator('text=Runtime layers')).toBeVisible();
    await expect(page.locator('text=Python')).toBeVisible();
    await expect(page.locator('text=Node.js')).toBeVisible();
  });

  test('navigates to settings section', async ({ page }) => {
    await page.goto('/');
    // Wait for initial load to finish
    await expect(page.locator('text=Workspace pulse')).toBeVisible();
    // Click Settings in the sidebar nav
    const settingsBtn = page.getByRole('button', { name: /Mirrors and export.*Settings/ });
    await settingsBtn.click();
    // The eyebrow text should change to "Mirrors and export"
    await expect(page.locator('text=Mirrors and export')).toBeVisible({ timeout: 10_000 });
  });

  test('job queue is visible in footer', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('text=Job Queue')).toBeVisible();
    await expect(page.locator('text=Runtime policy')).toBeVisible();
  });
});
