import { test, expect } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const testPlyFile = path.join(__dirname, "fixtures", "test.ply");
const truckPlyFile = path.join(__dirname, "fixtures", "truck.ply");

// Increase timeout for WASM-based tests
test.setTimeout(60000);

test.describe("PLY to SPLAT Converter", () => {
	test("should display the converter page", async ({ page }) => {
		await page.goto("/");

		// Check page title and main heading
		await expect(page).toHaveTitle(/Ply2Splat/);
		await expect(
			page.getByRole("heading", { name: "PLY to SPLAT Converter" }),
		).toBeVisible();

		// Check that the file upload area is present
		await expect(page.getByText("Click to upload")).toBeVisible();
		await expect(page.getByText("PLY files only")).toBeVisible();
	});

	test("should convert a PLY file and show success state", async ({
		page,
	}) => {
		await page.goto("/");

		// Get the hidden file input and upload the test file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(testPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 60000,
		});

		// Verify conversion stats are displayed
		await expect(page.getByText("Input file:")).toBeVisible();
		await expect(page.getByText("test.ply")).toBeVisible();
		await expect(page.getByText("Input size:")).toBeVisible();
		await expect(page.getByText("Splats:")).toBeVisible();
		await expect(page.getByText("Output size:")).toBeVisible();

		// Verify splat count is shown (should be 100 for our test file)
		// Locate the "Splats:" row and verify it contains "100"
		const splatsRow = page.locator("div").filter({ hasText: /^Splats:/ });
		await expect(splatsRow.getByText("100")).toBeVisible();
	});

	test("should show 3D preview after conversion", async ({ page }) => {
		await page.goto("/");

		// Upload the test file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(testPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 60000,
		});

		// Check that 3D Preview section is visible
		await expect(page.getByText("3D Preview")).toBeVisible();

		// Check that the canvas element exists (WebGL preview)
		const canvas = page.locator("canvas");
		await expect(canvas).toBeVisible({ timeout: 15000 });

		// Verify the preview help text is visible
		await expect(
			page.getByText("Use mouse to rotate • Scroll to zoom"),
		).toBeVisible();
	});

	test("should toggle preview visibility", async ({ page }) => {
		await page.goto("/");

		// Upload the test file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(testPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 60000,
		});

		// Verify preview is visible initially
		const canvas = page.locator("canvas");
		await expect(canvas).toBeVisible({ timeout: 15000 });

		// Click hide button
		await page.getByRole("button", { name: "Hide" }).click();

		// Canvas should be hidden
		await expect(canvas).not.toBeVisible();

		// Click show button
		await page.getByRole("button", { name: "Show" }).click();

		// Canvas should be visible again
		await expect(canvas).toBeVisible({ timeout: 15000 });
	});

	test("should download converted SPLAT file", async ({ page }) => {
		await page.goto("/");

		// Upload the test file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(testPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 60000,
		});

		// Set up download listener
		const downloadPromise = page.waitForEvent("download");

		// Click download button
		await page.getByRole("button", { name: "Download SPLAT" }).click();

		// Verify download is triggered
		const download = await downloadPromise;
		// The download uses the original filename, verify it's triggered
		expect(download.suggestedFilename()).toBeTruthy();

		// Verify file has content (100 splats * 32 bytes = 3200 bytes)
		const downloadPath = await download.path();
		expect(downloadPath).toBeTruthy();
	});

	test("should allow converting another file", async ({ page }) => {
		await page.goto("/");

		// First conversion
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(testPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 60000,
		});

		// Click "Convert another" button
		await page.getByRole("button", { name: "Convert another" }).click();

		// Verify we're back to the upload state
		await expect(page.getByText("Click to upload")).toBeVisible();
		await expect(page.getByText("Conversion complete!")).not.toBeVisible();
	});
});

// Separate test suite for real-world large PLY files
test.describe("Real-world PLY file conversion", () => {
	// Extended timeout for large file processing (486MB truck.ply with 2M+ vertices)
	test.setTimeout(300000);

	test("should convert truck.ply (real-world Gaussian Splatting file)", async ({
		page,
	}) => {
		await page.goto("/");

		// Upload the real-world truck PLY file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(truckPlyFile);

		// Wait for conversion to complete (extended timeout for large file)
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 300000,
		});

		// Verify conversion stats are displayed
		await expect(page.getByText("Input file:")).toBeVisible();
		await expect(page.getByText("truck.ply")).toBeVisible();
		await expect(page.getByText("Input size:")).toBeVisible();
		await expect(page.getByText("Splats:")).toBeVisible();
		await expect(page.getByText("Output size:")).toBeVisible();

		// Verify splat count is shown (should be 2,056,645 for the truck file)
		const splatsRow = page.locator("div").filter({ hasText: /^Splats:/ });
		await expect(splatsRow.getByText("2,056,645")).toBeVisible();
	});

	test("should show 3D preview for truck.ply", async ({ page }) => {
		await page.goto("/");

		// Upload the real-world truck PLY file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(truckPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 300000,
		});

		// Check that 3D Preview section is visible
		await expect(page.getByText("3D Preview")).toBeVisible();

		// Check that the canvas element exists (WebGL preview)
		const canvas = page.locator("canvas");
		await expect(canvas).toBeVisible({ timeout: 30000 });
	});

	test("should download converted truck.splat file", async ({ page }) => {
		await page.goto("/");

		// Upload the real-world truck PLY file
		const fileInput = page.locator('input[type="file"]');
		await fileInput.setInputFiles(truckPlyFile);

		// Wait for conversion to complete
		await expect(page.getByText("Conversion complete!")).toBeVisible({
			timeout: 300000,
		});

		// Set up download listener
		const downloadPromise = page.waitForEvent("download");

		// Click download button
		await page.getByRole("button", { name: "Download SPLAT" }).click();

		// Verify download is triggered
		const download = await downloadPromise;
		expect(download.suggestedFilename()).toBeTruthy();

		// Verify file path exists
		const downloadPath = await download.path();
		expect(downloadPath).toBeTruthy();
	});
});
