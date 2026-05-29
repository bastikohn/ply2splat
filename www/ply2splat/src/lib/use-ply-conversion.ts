import { convert, initWasm } from "@ply2splat/browser";
import { useCallback, useState } from "react";

export type ConversionStatus =
	| "idle"
	| "loading-wasm"
	| "converting"
	| "success"
	| "error";

export interface ConversionState {
	status: ConversionStatus;
	fileName: string | null;
	fileSize: number | null;
	splatCount: number | null;
	splatData: Uint8Array | null;
	error: string | null;
	progress: number;
}

const initialState: ConversionState = {
	status: "idle",
	fileName: null,
	fileSize: null,
	splatCount: null,
	splatData: null,
	error: null,
	progress: 0,
};

export function usePlyConversion() {
	const [state, setState] = useState<ConversionState>(initialState);

	const convertFile = useCallback(async (file: File) => {
		setState({
			status: "loading-wasm",
			fileName: file.name,
			fileSize: file.size,
			splatCount: null,
			splatData: null,
			error: null,
			progress: 5,
		});

		try {
			await initWasm();
			setState((prev) => ({ ...prev, status: "converting", progress: 30 }));

			const arrayBuffer = await file.arrayBuffer();
			setState((prev) => ({ ...prev, progress: 50 }));

			const plyData = new Uint8Array(arrayBuffer);
			setState((prev) => ({ ...prev, progress: 70 }));

			const result = await convert(plyData, { sort: false });
			setState((prev) => ({ ...prev, progress: 90 }));

			setState({
				status: "success",
				fileName: file.name,
				fileSize: file.size,
				splatCount: result.count,
				splatData: result.data,
				error: null,
				progress: 100,
			});
		} catch (err) {
			setState((prev) => ({
				...prev,
				status: "error",
				error: err instanceof Error ? err.message : "Unknown error occurred",
				progress: 0,
			}));
		}
	}, []);

	const selectFile = useCallback(
		(file: File | null | undefined) => {
			if (!file) return;
			if (!file.name.toLowerCase().endsWith(".ply")) {
				setState((prev) => ({
					...prev,
					status: "error",
					error: "Please select a PLY file",
				}));
				return;
			}
			void convertFile(file);
		},
		[convertFile],
	);

	const reset = useCallback(() => {
		setState(initialState);
	}, []);

	return { state, selectFile, reset };
}
