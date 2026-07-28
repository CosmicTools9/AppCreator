import { useState } from "react";
import { cn } from "../../lib/utils";

interface ColorSwatchProps {
  name: string;
  value: string;
  token: string;
  className?: string;
}

export function ColorSwatch({
  name,
  value,
  token,
  className,
}: ColorSwatchProps) {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = async () => {
    await navigator.clipboard.writeText(`var(${token})`);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Check if color is light or dark for text contrast
  const isLight = (hex: string) => {
    // Handle HSL format
    if (hex.startsWith("hsl")) {
      return true; // Default to dark text for HSL
    }
    // Handle hex format
    const cleanHex = hex.replace("#", "");
    if (cleanHex.length !== 6) return true;

    const r = parseInt(cleanHex.slice(0, 2), 16);
    const g = parseInt(cleanHex.slice(2, 4), 16);
    const b = parseInt(cleanHex.slice(4, 6), 16);
    const brightness = (r * 299 + g * 587 + b * 114) / 1000;
    return brightness > 128;
  };

  const textColor = isLight(value) ? "text-muted-foreground" : "text-white";

  return (
    <button
      onClick={copyToClipboard}
      className={cn(
        "group relative flex flex-col items-start gap-1 rounded-lg border border-border p-3 transition-transform transition-shadow hover:scale-105 hover:shadow-md",
        className,
      )}
      style={{ backgroundColor: value }}
    >
      <span className={cn("text-xs font-medium", textColor)}>{name}</span>
      <span className={cn("text-xs opacity-70", textColor)}>{value}</span>
      <span className={cn("text-xs opacity-50 font-mono", textColor)}>
        {token}
      </span>
      {copied && (
        <span className="absolute inset-0 flex items-center justify-center rounded-lg bg-black/50 text-white text-xs font-medium">
          Copied!
        </span>
      )}
    </button>
  );
}
