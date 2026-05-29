import { CheckCircle2, Download, Eye, EyeOff, Loader2 } from "lucide-react";
import { Suspense, useState } from "react";
import { SplatPreview } from "@/components/SplatPreview";
import { Button } from "@/components/ui/button";
import { formatFileSize } from "@/lib/size-conversion";
import { useDownloadSplat } from "@/lib/use-download-data";
import type { ConversionState } from "@/lib/use-ply-conversion";

interface ConversionResultPanelProps {
	state: ConversionState;
	onReset: () => void;
}

export function ConversionResultPanel({
	state,
	onReset,
}: ConversionResultPanelProps) {
	const [showPreview, setShowPreview] = useState(true);
	const { downloadSplat } = useDownloadSplat(state.splatData, state.fileName);

	return (
		<div className="max-w-xl mx-auto rounded-lg border bg-card p-6">
			<div className="py-4 space-y-6">
				<div className="flex items-center justify-center gap-2 text-green-600 dark:text-green-500">
					<CheckCircle2 className="h-6 w-6" />
					<span className="font-medium">Conversion complete!</span>
				</div>

				{state.splatData && (
					<div className="space-y-2">
						<div className="flex items-center justify-between">
							<span className="text-sm font-medium">3D Preview</span>
							<Button
								variant="ghost"
								size="sm"
								onClick={() => setShowPreview((value) => !value)}
								className="h-8"
							>
								{showPreview ? (
									<>
										<EyeOff className="h-4 w-4" />
										Hide
									</>
								) : (
									<>
										<Eye className="h-4 w-4" />
										Show
									</>
								)}
							</Button>
						</div>
						{showPreview && (
							<>
								<Suspense
									fallback={
										<div className="w-full h-[400px] rounded-lg bg-muted/50 flex items-center justify-center">
											<Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
										</div>
									}
								>
									<SplatPreview splatData={state.splatData} />
								</Suspense>
								<p className="text-xs text-muted-foreground text-center">
									Use mouse to rotate • Scroll to zoom
								</p>
							</>
						)}
					</div>
				)}

				<div className="bg-muted/50 rounded-lg p-4 space-y-2">
					<div className="flex justify-between text-sm">
						<span className="text-muted-foreground">Input file:</span>
						<span className="font-medium">{state.fileName}</span>
					</div>
					<div className="flex justify-between text-sm">
						<span className="text-muted-foreground">Input size:</span>
						<span className="font-medium">
							{state.fileSize ? formatFileSize(state.fileSize) : "-"}
						</span>
					</div>
					<div className="flex justify-between text-sm">
						<span className="text-muted-foreground">Splats:</span>
						<span className="font-medium">
							{state.splatCount?.toLocaleString()}
						</span>
					</div>
					<div className="flex justify-between text-sm">
						<span className="text-muted-foreground">Output size:</span>
						<span className="font-medium">
							{state.splatData ? formatFileSize(state.splatData.length) : "-"}
						</span>
					</div>
				</div>

				<div className="flex gap-3">
					<Button onClick={downloadSplat} className="flex-1">
						<Download className="h-4 w-4" />
						Download SPLAT
					</Button>
					<Button variant="outline" onClick={onReset}>
						Convert another
					</Button>
				</div>
			</div>
		</div>
	);
}
