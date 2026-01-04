import { Canvas } from "@react-three/fiber";
import { Splat, OrbitControls } from "@react-three/drei";
import { Suspense, useMemo, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";

interface SplatPreviewProps {
  splatData: Uint8Array;
}

function SplatScene({ blobUrl }: { blobUrl: string }) {
  return (
    <>
      <OrbitControls makeDefault />
      <Suspense fallback={null}>
        <Splat src={blobUrl} />
      </Suspense>
    </>
  );
}

export function SplatPreview({ splatData }: SplatPreviewProps) {
  const [isLoading, setIsLoading] = useState(true);
  const blobUrl = useMemo(() => {
    // Copy the data to handle SharedArrayBuffer case
    const data = new Uint8Array(splatData);
    const blob = new Blob([data], { type: "application/octet-stream" });
    return URL.createObjectURL(blob);
  }, [splatData]);

  // Cleanup blob URL when component unmounts or data changes
  useEffect(() => {
    return () => {
      URL.revokeObjectURL(blobUrl);
    };
  }, [blobUrl]);

  return (
    <div className="relative w-full h-[400px] rounded-lg overflow-hidden bg-black/90">
      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center z-10">
          <Loader2 className="h-8 w-8 animate-spin text-white/50" />
        </div>
      )}
      <Canvas
        camera={{ position: [0, 0, 5], fov: 50 }}
        onCreated={() => setIsLoading(false)}
      >
        <SplatScene blobUrl={blobUrl} />
      </Canvas>
    </div>
  );
}
