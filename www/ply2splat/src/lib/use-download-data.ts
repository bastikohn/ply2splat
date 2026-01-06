import * as React from "react";

export const downloadData = (
	url: string,
	options: {
		fileName?: string;
		onCleanup?: () => void;
	} = {},
) => {
	const a = document.createElement("a");
	a.href = url;
	a.download = options.fileName ?? url;
	document.body.appendChild(a);
	a.click();

	// Delay cleanup to ensure download initiates properly
	// Remove anchor element asynchronously to avoid layout thrashing
	setTimeout(() => {
		document.body.removeChild(a);
		// Call cleanup callback after anchor is removed
		options.onCleanup?.();
	}, 100);
};

export const useDownloadSplat = (
	splatData: Uint8Array<ArrayBufferLike> | null,
	fileName: string | null,
) => {
	const downloadSplat = React.useCallback(() => {
		if (!splatData || !fileName) return;

		// Copy the data to a regular ArrayBuffer (handles SharedArrayBuffer case)
		const data = new Uint8Array(splatData);
		const blob = new Blob([data], { type: "application/octet-stream" });
		const url = URL.createObjectURL(blob);

		downloadData(url, {
			fileName,
			onCleanup: () => {
				// Revoke URL after download has had time to start
				URL.revokeObjectURL(url);
			},
		});
	}, [splatData, fileName]);
	return { downloadSplat };
};
