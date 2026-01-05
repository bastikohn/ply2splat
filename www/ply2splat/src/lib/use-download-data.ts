import * as React from "react";

export const downloadData = (
	url: string,
	options: {
		fileName?: string;
	} = {},
) => {
	const a = document.createElement("a");
	a.href = url;
	a.download = options?.fileName ?? url;
	document.body.appendChild(a);
	a.click();
	document.body.removeChild(a);
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
		downloadData(url, { fileName });
		URL.revokeObjectURL(url);
	}, [splatData, fileName]);
	return { downloadSplat };
};
