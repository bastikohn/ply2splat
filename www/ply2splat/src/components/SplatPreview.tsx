import { Canvas } from "@react-three/fiber";
import { CameraControls, Splat } from "@react-three/drei";
import { Suspense, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";

interface SplatPreviewProps {
	splatData: Uint8Array;
}

function SplatScene({ splatUrl }: { splatUrl: string }) {
	return (
		<>
			<CameraControls infinityDolly minDistance={0.1} />
			<Suspense fallback={null}>
				<Splat src={splatUrl} />
			</Suspense >
		</>
	);
}

export function SplatPreview({ splatData }: SplatPreviewProps) {
	const [isLoading, setIsLoading] = useState(true);
	const [splatUrl, setSplatUrl] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		// Upload splat data to the preview server endpoint
		// This avoids blob: URL issues in cross-origin isolated contexts
		const uploadSplatData = async () => {
			try {
				setIsLoading(true);
				setError(null);

				const data = new Uint8Array(splatData);
				const blob = new Blob([data]);
				const url = URL.createObjectURL(blob);
				setSplatUrl(url);
			} catch (err) {
				console.error("Failed to prepare splat preview:", err);
				setError(err instanceof Error ? err.message : "Failed to load preview");
				setIsLoading(false);
			}
		};

		uploadSplatData();
	}, [splatData]);

	if (error) {
		return (
			<div className="w-full h-[400px] rounded-lg bg-black/90 flex items-center justify-center">
				<p className="text-red-400 text-sm">{error}</p>
			</div>
		);
	}

	return (
		<div className="relative w-full h-[400px] rounded-lg overflow-hidden bg-black/90">
			{(isLoading || !splatUrl) && (
				<div className="absolute inset-0 flex items-center justify-center z-10">
					<Loader2 className="h-8 w-8 animate-spin text-white/50" />
				</div>
			)}
			{splatUrl && (
				<Canvas
					onCreated={() => setIsLoading(false)}
				>
					<SplatScene splatUrl={splatUrl} />
				</Canvas>
			)}
		</div>
	);
}
