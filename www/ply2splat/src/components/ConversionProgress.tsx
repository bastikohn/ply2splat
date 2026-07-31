import { Loader2 } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import type { ConversionState } from "@/lib/use-ply-conversion";

interface ConversionProgressProps {
	state: ConversionState;
}

export function ConversionProgress({ state }: ConversionProgressProps) {
	return (
		<div className="max-w-xl mx-auto rounded-lg border bg-card p-6">
			<div className="py-8 space-y-4">
				<div className="flex items-center justify-center gap-3">
					<Loader2 className="h-5 w-5 animate-spin text-primary" />
					<span className="text-sm font-medium">
						{state.status === "loading-wasm"
							? "Loading converter..."
							: "Converting..."}
					</span>
				</div>
				<Progress value={state.progress} />
				{state.fileName && (
					<p className="text-center text-sm text-muted-foreground">
						{state.fileName}
					</p>
				)}
			</div>
		</div>
	);
}
