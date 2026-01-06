import { test, expect } from '@playwright/test';

test.describe('Landing Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('loads successfully', async ({ page }) => {
    // Page should load without errors
    await expect(page).toHaveTitle(/Meridian/i);
  });

  test('displays hero section', async ({ page }) => {
    // Should have main heading
    const heading = page.getByRole('heading', { level: 1 });
    await expect(heading).toBeVisible();
  });

  test('has navigation to portal', async ({ page }) => {
    // Should have link to portal/login
    const loginLink = page.getByRole('link', { name: /login|portal|get started/i });
    await expect(loginLink.first()).toBeVisible();
  });

  test('has reserves link', async ({ page }) => {
    // Should have link to reserves page
    const reservesLink = page.getByRole('link', { name: /reserves/i });
    await expect(reservesLink.first()).toBeVisible();
  });

  test('is responsive on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.reload();

    // Page should still render without horizontal scroll
    const body = page.locator('body');
    const bodyWidth = await body.evaluate(el => el.scrollWidth);
    expect(bodyWidth).toBeLessThanOrEqual(375);
  });
});
