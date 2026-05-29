import { describe, expect, test } from "vitest";
import { formatFileSize } from "./size-conversion";

describe("formatFileSize", () => {
	test("formats bytes, kilobytes, and megabytes", () => {
		expect(formatFileSize(512)).toBe("512 B");
		expect(formatFileSize(1536)).toBe("1.5 KB");
		expect(formatFileSize(2 * 1024 * 1024)).toBe("2.0 MB");
	});
});
