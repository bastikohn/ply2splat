import { Canvas } from "@react-three/fiber";
import { Splat, OrbitControls } from "@react-three/drei";
import { Suspense, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";

interface SplatPreviewProps {
  splatData: Uint8Array;
}

function SplatScene({ splatUrl }: { splatUrl: string }) {
  return (
    <>
      <OrbitControls makeDefault />
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
    // Upload splat data to the preview server endpoint
    // This avoids blob: URL issues in cross-origin isolated contexts
    const uploadSplatData = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // Copy data to ensure we have a clean ArrayBuffer
        const data = new Uint8Array(splatData);

        const response = await fetch("/__splat_preview", {
          method: "POST",
          body: data,
          headers: {
            "Content-Type": "application/octet-stream",
          },
        });

        if (!response.ok) {
          throw new Error(`Failed to upload splat data: ${response.status}`);
        }

        const { id } = await response.json();
        setSplatUrl(`/__splat_preview?id=${id}`);
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
          camera={{ position: [0, 0, 5], fov: 50 }}
          onCreated={() => setIsLoading(false)}
        >
          <SplatScene splatUrl={splatUrl} />
        </Canvas>
      )}
    </div>
  );
}
