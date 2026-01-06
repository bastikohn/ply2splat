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
			</Suspense>
		</>
	);
}

export function SplatPreview({ splatData }: SplatPreviewProps) {
	const [isLoading, setIsLoading] = useState(true);
	const [splatUrl, setSplatUrl] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		setIsLoading(true);
		setError(null);

		let url: string | null = null;
		try {
			// Create a copy to ensure proper ArrayBuffer type for Blob constructor
			const blob = new Blob([splatData.slice()], {
				type: "application/octet-stream",
			});
			url = URL.createObjectURL(blob);
			console.log("Created URL:", url);
			setSplatUrl(url);
		} catch (err) {
			console.error("Failed to create blob URL:", err);
			setError(err instanceof Error ? err.message : "Failed to load preview");
			setSplatUrl(null);
		}

		// Cleanup blob URL when component unmounts or splatData changes
		return () => {
			if (url) {
				console.log("Revoking url", url);
				URL.revokeObjectURL(url);
			}
		};
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
					camera={{ near: 0.001, fov: 30 }}
					onCreated={() => setIsLoading(false)}
				>
					<SplatScene splatUrl={splatUrl} />
				</Canvas>
			)}
		</div>
	);
}
