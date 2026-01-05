import { createFileRoute } from "@tanstack/react-router";
import { useState, useCallback, useRef, lazy, Suspense } from "react";
import {
  Upload,
  Download,
  FileType2,
  Loader2,
  CheckCircle2,
  AlertCircle,
  Eye,
  EyeOff,
} from "lucide-react";
import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Progress } from "../components/ui/progress";
import { initWasm, convert } from "../lib/ply2splat-client";
import { useDownloadSplat } from "@/lib/use-download-data";
import { SplatPreview } from "../components/SplatPreview";
import { formatFileSize } from "@/lib/size-conversion";

export const Route = createFileRoute("/")({
  component: App,
});

type ConversionStatus =
  | "idle"
  | "loading-wasm"
  | "converting"
  | "success"
  | "error";

interface ConversionState {
  status: ConversionStatus;
  fileName: string | null;
  fileSize: number | null;
  splatCount: number | null;
  splatData: Uint8Array | null;
  error: string | null;
  progress: number;
}


function App() {
  const [state, setState] = useState<ConversionState>({
    status: "idle",
    fileName: null,
    fileSize: null,
    splatCount: null,
    splatData: null,
    error: null,
    progress: 0,
  });

  const [isDragging, setIsDragging] = useState(false);
  const [showPreview, setShowPreview] = useState(true);
  const fileInputRef = useRef<HTMLInputElement>(null);

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

      const result = await convert(plyData, false);
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

  const { downloadSplat } = useDownloadSplat(state.splatData, state.fileName);

  const handleFileSelect = useCallback(
    (files: FileList | null) => {
      if (!files || files.length === 0) return;
      const file = files[0];
      if (!file.name.toLowerCase().endsWith(".ply")) {
        setState((prev) => ({
          ...prev,
          status: "error",
          error: "Please select a PLY file",
        }));
        return;
      }
      convertFile(file);
    },
    [convertFile],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragging(false);
      handleFileSelect(e.dataTransfer.files);
    },
    [handleFileSelect],
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const reset = useCallback(() => {
    setState({
      status: "idle",
      fileName: null,
      fileSize: null,
      splatCount: null,
      splatData: null,
      error: null,
      progress: 0,
    });
    setShowPreview(true);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  }, []);

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
          {state.status === "idle" || state.status === "error" ? (
            <>
              <div
                onClick={() => fileInputRef.current?.click()}
                onDrop={handleDrop}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    fileInputRef.current?.click();
                  }
                }}
                role="button"
                tabIndex={0}
                className={`
                  relative border-2 border-dashed rounded-lg p-12 text-center cursor-pointer
                  transition-colors duration-200
                  ${isDragging
                    ? "border-primary bg-primary/5"
                    : "border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50"
                  }
                `}
              >
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".ply"
                  onChange={(e) => handleFileSelect(e.target.files)}
                  className="hidden"
                />
                <Upload className="h-10 w-10 mx-auto mb-4 text-muted-foreground" />
                <p className="text-sm text-muted-foreground">
                  <span className="font-medium text-foreground">
                    Click to upload
                  </span>{" "}
                  or drag and drop
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  PLY files only
                </p>
              </div>

              {state.status === "error" && state.error && (
                <div className="flex items-center gap-2 text-destructive text-sm bg-destructive/10 p-3 rounded-lg">
                  <AlertCircle className="h-4 w-4 shrink-0" />
                  <span>{state.error}</span>
                </div>
              )}
            </>
          ) : state.status === "loading-wasm" ||
            state.status === "converting" ? (
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
          ) : state.status === "success" ? (
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
                      onClick={() => setShowPreview(!showPreview)}
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
                    {state.splatData
                      ? formatFileSize(state.splatData.length)
                      : "-"}
                  </span>
                </div>
              </div>

              <div className="flex gap-3">
                <Button onClick={downloadSplat} className="flex-1">
                  <Download className="h-4 w-4" />
                  Download SPLAT
                </Button>
                <Button variant="outline" onClick={reset}>
                  Convert another
                </Button>
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>

      <div className="mt-16 grid gap-8 md:grid-cols-3 text-center">
        <div>
          <div className="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 mb-4">
            <svg
              className="h-6 w-6 text-primary"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>
          </div>
          <h3 className="font-semibold mb-2">Fast Conversion</h3>
          <p className="text-sm text-muted-foreground">
            Powered by WebAssembly for near-native performance
          </p>
        </div>
        <div>
          <div className="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 mb-4">
            <svg
              className="h-6 w-6 text-primary"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
              />
            </svg>
          </div>
          <h3 className="font-semibold mb-2">100% Private</h3>
          <p className="text-sm text-muted-foreground">
            All processing happens locally in your browser
          </p>
        </div>
        <div>
          <div className="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 mb-4">
            <svg
              className="h-6 w-6 text-primary"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"
              />
            </svg>
          </div>
          <h3 className="font-semibold mb-2">Smaller Files</h3>
          <p className="text-sm text-muted-foreground">
            SPLAT format is optimized for web viewing
          </p>
        </div>
      </div>
    </div>
  );
}
