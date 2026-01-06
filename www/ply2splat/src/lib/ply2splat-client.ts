/**
 * PLY to SPLAT conversion client for the web app.
 *
 * Uses @ply2splat/browser with Vite-resolved asset URLs.
 */

import {
	createClient,
	type Ply2SplatClient,
	type ConversionResult,
} from "@ply2splat/browser";

// Re-export types for consumers
export type { ConversionResult };

// Singleton client instance
let client: Ply2SplatClient | null = null;

function getClient(): Ply2SplatClient {
	if (!client) {
		client = createClient();
	}
	return client;
}

/**
 * Initialize the WASM module.
 * This is optional - convert() will auto-initialize if needed.
 */
export async function initWasm(): Promise<void> {
	await getClient().initWasm();
}

/**
 * Convert PLY data to SPLAT format.
 *
 * @param plyData - PLY file contents as a Uint8Array
 * @param sort - Whether to sort splats by importance (default: true)
 * @returns Promise resolving to the conversion result
 */
export async function convert(
	plyData: Uint8Array,
	sort: boolean = true,
): Promise<ConversionResult> {
	return getClient().convert(plyData, { sort });
}

/**
 * Terminate the worker and clean up resources.
 */
export function terminate(): void {
	if (client) {
		client.terminate();
		client = null;
	}
}
