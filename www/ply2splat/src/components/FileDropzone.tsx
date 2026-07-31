import { AlertCircle, FileType2, Upload } from "lucide-react";
import type { DragEvent } from "react";
import { useCallback, useRef, useState } from "react";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";

interface FileDropzoneProps {
	error: string | null;
	onFileSelected: (file: File | null | undefined) => void;
}

export function FileDropzone({ error, onFileSelected }: FileDropzoneProps) {
	const [isDragging, setIsDragging] = useState(false);
	const fileInputRef = useRef<HTMLInputElement>(null);

	const openFilePicker = useCallback(() => {
		if (fileInputRef.current) {
			fileInputRef.current.value = "";
			fileInputRef.current.click();
		}
	}, []);

	const handleDrop = useCallback(
		(e: DragEvent) => {
			e.preventDefault();
			setIsDragging(false);
			onFileSelected(e.dataTransfer.files[0]);
		},
		[onFileSelected],
	);

	const handleDragOver = useCallback((e: DragEvent) => {
		e.preventDefault();
		setIsDragging(true);
	}, []);

	const handleDragLeave = useCallback((e: DragEvent) => {
		e.preventDefault();
		setIsDragging(false);
	}, []);

	return (
		<Card className="max-w-xl mx-auto">
			<CardHeader>
				<CardTitle className="flex items-center gap-2">
					<FileType2 className="h-5 w-5" />
					File Converter
				</CardTitle>
				<CardDescription>
					Drop your PLY file here or click to browse
				</CardDescription>
			</CardHeader>
			<CardContent className="space-y-6">
				<input
					ref={fileInputRef}
					type="file"
					accept=".ply"
					onChange={(e) => onFileSelected(e.target.files?.[0])}
					className="hidden"
				/>
				<button
					type="button"
					onClick={openFilePicker}
					onDrop={handleDrop}
					onDragOver={handleDragOver}
					onDragLeave={handleDragLeave}
					className={`
						relative w-full border-2 border-dashed rounded-lg p-12 text-center cursor-pointer
						transition-colors duration-200
						${
							isDragging
								? "border-primary bg-primary/5"
								: "border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50"
						}
					`}
				>
					<Upload className="h-10 w-10 mx-auto mb-4 text-muted-foreground" />
					<p className="text-sm text-muted-foreground">
						<span className="font-medium text-foreground">Click to upload</span>{" "}
						or drag and drop
					</p>
					<p className="text-xs text-muted-foreground mt-1">PLY files only</p>
				</button>

				{error && (
					<div className="flex items-center gap-2 text-destructive text-sm bg-destructive/10 p-3 rounded-lg">
						<AlertCircle className="h-4 w-4 shrink-0" />
						<span>{error}</span>
					</div>
				)}
			</CardContent>
		</Card>
	);
}
