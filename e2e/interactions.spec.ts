import { test, expect } from '@playwright/test';

test.describe('Forge Env — interaction tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('text=Workspace pulse')).toBeVisible();
  });

  test('theme toggle switches between light and dark', async ({ page }) => {
    const html = page.locator('html');
    // Default should be light or dark based on system
    const initialTheme = await html.getAttribute('data-theme');

    // Find and click the theme toggle button
    const toggle = page.getByRole('button', { name: /Switch to/ });
    await toggle.click();

    // Theme should have changed
    const newTheme = await html.getAttribute('data-theme');
    expect(newTheme).not.toBe(initialTheme);

    // Toggle back
    await toggle.click();
    const restoredTheme = await html.getAttribute('data-theme');
    expect(restoredTheme).toBe(initialTheme);
  });

  test('overview shows sidebar with metrics', async ({ page }) => {
    // Sidebar should be visible with Live Status
    await expect(page.locator('aside').getByText('Live Status')).toBeVisible();
  });

  test('overview mirror preset selector works', async ({ page }) => {
    const select = page.locator('select').first();
    await expect(select).toBeVisible();

    // Should have mirror options
    await expect(select.locator('option:has-text("Tsinghua")')).toBeAttached();
    await expect(select.locator('option:has-text("Aliyun")')).toBeAttached();
  });

  test('hosts section shows host cards', async ({ page }) => {
    const hostsBtn = page.locator('nav button', { hasText: 'Hosts' });
    await hostsBtn.click();

    await expect(page.locator('text=Browser Preview Host')).toBeVisible();
  });

  test('hosts section allows host selection', async ({ page }) => {
    const hostsBtn = page.locator('nav button', { hasText: 'Hosts' });
    await hostsBtn.click();

    // Click the WSL host
    const wslCard = page.locator('button', { hasText: 'WSL Distro' });
    await wslCard.click();

    // Detail panel should update
    await expect(page.locator('text=Host detail')).toBeVisible();
  });

  test('languages section shows runtime inventory', async ({ page }) => {
    const langBtn = page.locator('nav button', { hasText: 'Languages' });
    await langBtn.click();

    await expect(page.locator('section').getByText('Python', { exact: true })).toBeVisible();
    await expect(page.locator('section').getByText('Node.js', { exact: true })).toBeVisible();
    await expect(page.locator('section').getByText('Rust', { exact: true })).toBeVisible();
    await expect(page.locator('section').getByText('Java', { exact: true })).toBeVisible();
  });

  test('languages section shows installed versions', async ({ page }) => {
    const langBtn = page.locator('nav button', { hasText: 'Languages' });
    await langBtn.click();

    // Should show version numbers
    await expect(page.locator('text=3.12.4')).toBeVisible();
    await expect(page.locator('text=Installed versions')).toBeVisible();
  });

  test('projects section has search filter', async ({ page }) => {
    const projBtn = page.locator('nav button', { hasText: 'Projects' });
    await projBtn.click();

    const searchInput = page.locator('input[placeholder*="Filter"]');
    await expect(searchInput).toBeVisible();

    // Type in search
    await searchInput.fill('python');
    // Projects should be filtered
    await expect(page.locator('text=Marker-driven runtime suggestions')).toBeVisible();
  });

  test('system deps section shows services', async ({ page }) => {
    const depsBtn = page.locator('nav button', { hasText: 'System Deps' });
    await depsBtn.click();

    await expect(page.locator('section').getByText('Service provider MVP')).toBeVisible();
    await expect(page.locator('section').getByText('Redis', { exact: true })).toBeVisible();
    await expect(page.locator('section').getByText('PostgreSQL', { exact: true })).toBeVisible();
  });

  test('settings section shows proxy and export panels', async ({ page }) => {
    const settingsBtn = page.locator('nav button', { hasText: 'Settings' });
    await settingsBtn.click();

    await expect(page.locator('text=Secure proxy profile')).toBeVisible();
    await expect(page.locator('text=Export template')).toBeVisible();
    await expect(page.locator('text=Import reconstruction')).toBeVisible();
  });

  test('job queue shows filter buttons', async ({ page }) => {
    await expect(page.locator('text=Job Queue')).toBeVisible();
    await expect(page.locator('button:has-text("Relevant")')).toBeVisible();
    await expect(page.locator('button:has-text("All jobs")')).toBeVisible();
  });

  test('sidebar status shows ready state', async ({ page }) => {
    await expect(page.locator('aside').getByText('Live Status')).toBeVisible();
    await expect(page.locator('aside').getByText('Ready')).toBeVisible();
  });

  test('refresh button triggers data reload', async ({ page }) => {
    const refreshBtn = page.locator('button:has-text("Refresh snapshot")');
    await expect(refreshBtn).toBeVisible();
    await refreshBtn.click();
    // Should still show data after refresh
    await expect(page.locator('text=Workspace pulse')).toBeVisible();
  });
});
