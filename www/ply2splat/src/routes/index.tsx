import { createFileRoute } from "@tanstack/react-router";
import { Database, LockKeyhole, type LucideIcon, Zap } from "lucide-react";
import { ConversionProgress } from "@/components/ConversionProgress";
import { ConversionResultPanel } from "@/components/ConversionResultPanel";
import { FileDropzone } from "@/components/FileDropzone";
import { usePlyConversion } from "@/lib/use-ply-conversion";

export const Route = createFileRoute("/")({
	component: App,
});

const features: Array<{
	title: string;
	description: string;
	icon: LucideIcon;
}> = [
	{
		title: "Fast Conversion",
		description: "Powered by WebAssembly for near-native performance",
		icon: Zap,
	},
	{
		title: "100% Private",
		description: "All processing happens locally in your browser",
		icon: LockKeyhole,
	},
	{
		title: "Smaller Files",
		description: "SPLAT format is optimized for web viewing",
		icon: Database,
	},
];

function App() {
	const { state, selectFile, reset } = usePlyConversion();

	return (
		<div className="container mx-auto max-w-4xl px-4 py-12">
			<div className="text-center mb-12">
				<h1 className="text-4xl font-bold tracking-tight mb-4">
					PLY to SPLAT Converter
				</h1>
				<p className="text-lg text-muted-foreground max-w-2xl mx-auto">
					Convert 3D Gaussian Splatting PLY files to the optimized SPLAT format.
					Fast, private, and runs entirely in your browser.
				</p>
			</div>

			{(state.status === "idle" || state.status === "error") && (
				<FileDropzone error={state.error} onFileSelected={selectFile} />
			)}

			{(state.status === "loading-wasm" || state.status === "converting") && (
				<ConversionProgress state={state} />
			)}

			{state.status === "success" && (
				<ConversionResultPanel state={state} onReset={reset} />
			)}

			<FeatureGrid />
		</div>
	);
}

function FeatureGrid() {
	return (
		<div className="mt-16 grid gap-8 md:grid-cols-3 text-center">
			{features.map(({ title, description, icon: Icon }) => (
				<div key={title}>
					<div className="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 mb-4">
						<Icon className="h-6 w-6 text-primary" />
					</div>
					<h3 className="font-semibold mb-2">{title}</h3>
					<p className="text-sm text-muted-foreground">{description}</p>
				</div>
			))}
		</div>
	);
}
