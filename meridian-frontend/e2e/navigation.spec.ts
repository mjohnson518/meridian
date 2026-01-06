import { test, expect } from '@playwright/test';

test.describe('Navigation', () => {
  test('landing page to reserves navigation', async ({ page }) => {
    await page.goto('/');

    // Find and click reserves link
    const reservesLink = page.getByRole('link', { name: /reserves/i });
    if (await reservesLink.count() > 0) {
      await reservesLink.first().click();
      await expect(page).toHaveURL(/reserves/);
    }
  });

  test('landing page to login navigation', async ({ page }) => {
    await page.goto('/');

    // Find and click login/portal link
    const loginLink = page.getByRole('link', { name: /login|portal|get started/i });
    if (await loginLink.count() > 0) {
      await loginLink.first().click();
      await page.waitForURL(/login|portal/);
    }
  });

  test('back button works correctly', async ({ page }) => {
    await page.goto('/');
    const initialUrl = page.url();

    // Navigate to reserves
    await page.goto('/reserves');
    await expect(page).toHaveURL(/reserves/);

    // Go back
    await page.goBack();
    await expect(page).toHaveURL(initialUrl);
  });
});

test.describe('Performance', () => {
  test('landing page loads within acceptable time', async ({ page }) => {
    const start = Date.now();
    await page.goto('/');
    const loadTime = Date.now() - start;

    // Page should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });

  test('reserves page loads within acceptable time', async ({ page }) => {
    const start = Date.now();
    await page.goto('/reserves');
    const loadTime = Date.now() - start;

    // Page should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });
});

test.describe('Error Handling', () => {
  test('404 page for invalid routes', async ({ page }) => {
    const response = await page.goto('/this-route-does-not-exist');

    // Should return 404 or redirect to home
    const status = response?.status();
    // Next.js may return 200 with 404 page or actual 404
    expect([200, 404]).toContain(status);

    // Should show some indication of not found or redirect
    const notFoundText = page.getByText(/not found|404|page.*exist/i);
    const isNotFound = await notFoundText.count() > 0;
    const redirectedHome = page.url().endsWith('/');

    expect(isNotFound || redirectedHome).toBeTruthy();
  });

  test('handles JavaScript errors gracefully', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', err => errors.push(err.message));

    await page.goto('/');
    await page.waitForTimeout(2000);

    // Should not have uncaught errors
    expect(errors.length).toBe(0);
  });
});

test.describe('Accessibility', () => {
  test('landing page has proper heading structure', async ({ page }) => {
    await page.goto('/');

    // Should have exactly one h1
    const h1Count = await page.locator('h1').count();
    expect(h1Count).toBe(1);
  });

  test('interactive elements are keyboard accessible', async ({ page }) => {
    await page.goto('/');

    // Tab through the page
    await page.keyboard.press('Tab');

    // Should have focused element
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('images have alt text', async ({ page }) => {
    await page.goto('/');

    // All images should have alt attribute
    const imagesWithoutAlt = await page.locator('img:not([alt])').count();
    expect(imagesWithoutAlt).toBe(0);
  });
});
