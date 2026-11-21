// SPDX-License-Identifier: MIT OR Apache-2.0
export type SourceFormat = "png" | "svg";
export type SourceShape = "landscape" | "portrait" | "square";

export interface SourceDimensions {
  width: number;
  height: number;
}

export interface PngSourceInfo {
  pixelCount: number;
  bitsPerChannel: number;
  hasAlphaChannel: boolean;
  hasTransparentPixels: boolean;
  hasIccProfile: boolean;
}

export type SourceWarning =
  | { code: "embedded-image-ignored" }
  | { code: "external-image-ignored" }
  | { code: "external-reference-ignored" }
  | { code: "external-style-ignored" }
  | { code: "font-definition-ignored" }
  | { code: "icc-profile-treated-as-srgb" }
  | { code: "non-square-raster"; width: number; height: number }
  | { code: "non-square-view-box"; width: number; height: number }
  | { code: "text-ignored" }
  | {
      code: "unsupported-svg-feature-ignored";
      feature: "animation" | "audio" | "canvas" | "iframe" | "link" | "video";
    };

export interface SourceInfo {
  role: string;
  path: string;
  format: SourceFormat;
  byteLength: number;
  contentHash: string;
  dimensions: SourceDimensions;
  shape: SourceShape;
  warnings: SourceWarning[];
  png: PngSourceInfo | null;
}

export interface PreviewImage {
  contentHash: string;
  width: number;
  height: number;
  pngBytes: number[];
  cacheHit: boolean;
}

export function sourceWarningMessage(warning: SourceWarning): string {
  switch (warning.code) {
    case "embedded-image-ignored":
      return "An embedded image was ignored. Flatten it into vector artwork before export.";
    case "external-image-ignored":
      return "An externally linked image was ignored; source files cannot read other files.";
    case "external-reference-ignored":
      return "An external SVG reference was ignored.";
    case "external-style-ignored":
      return "An external stylesheet was ignored.";
    case "font-definition-ignored":
      return "An SVG font definition was ignored. Convert text to paths for deterministic output.";
    case "icc-profile-treated-as-srgb":
      return "The PNG ICC profile is detected but v1 treats channel values as sRGB.";
    case "non-square-raster":
      return `The raster is ${warning.width}×${warning.height}; platform icon masters must be square.`;
    case "non-square-view-box":
      return `The SVG viewBox is ${warning.width}×${warning.height}; square icon outputs may distort.`;
    case "text-ignored":
      return "SVG text is ignored for deterministic output. Convert text to paths.";
    case "unsupported-svg-feature-ignored":
      return `The SVG ${warning.feature} feature is not preserved in static output.`;
  }
}

export function sourceOperationErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (!error || typeof error !== "object") return "The source operation failed.";

  const value = error as Record<string, unknown>;
  if (value.error) return sourceOperationErrorMessage(value.error);

  switch (value.kind) {
    case "invalid-role":
      return "The source role is invalid.";
    case "file-too-large":
      return "The source exceeds the 64 MiB file limit.";
    case "not-regular-file":
      return "Select a PNG or SVG file, not a directory.";
    case "unsupported-format":
      return "Only content-verified PNG and SVG sources are supported.";
    case "unsafe-svg":
      return unsafeSvgErrorMessage(value.violation);
    case "invalid-svg":
      return "The SVG is malformed or cannot be normalized safely.";
    case "invalid-png":
    case "png-decode":
      return "The PNG is malformed or cannot be decoded safely.";
    case "png-pixel-limit":
    case "png-dimension-limit":
    case "png-decoded-memory-limit":
      return "The PNG exceeds the safe image limits.";
    case "upscale-forbidden":
      return "This raster preview would upscale the original source.";
    case "dimension-limit":
    case "pixel-limit":
      return "The requested preview dimensions exceed the safe preview limits.";
    case "cache-unavailable":
    case "cache-invariant":
      return "The preview cache is unavailable. Restart the application and try again.";
    default:
      return typeof value.message === "string" && value.message
        ? value.message
        : "The source operation failed.";
  }
}

function unsafeSvgErrorMessage(violation: unknown): string {
  const fallback = "The SVG contains an active or resource-amplifying construct and was rejected.";
  if (!violation || typeof violation !== "object") return fallback;

  const value = violation as Record<string, unknown>;
  switch (value.code) {
    case "document-type":
      return "SVG document type and entity declarations are forbidden.";
    case "event-handler-attribute":
      return typeof value.attribute === "string"
        ? `SVG event handler ${value.attribute} is forbidden.`
        : "SVG event-handler attributes are forbidden.";
    case "foreign-object-element":
      return "SVG foreignObject elements are forbidden.";
    case "maximum-element-count":
      return "The SVG exceeds the safe element-count limit.";
    case "maximum-nesting-depth":
      return "The SVG exceeds the safe nesting-depth limit.";
    case "script-element":
      return "SVG script elements are forbidden.";
    case "script-reference":
      return "SVG script references are forbidden.";
    default:
      return fallback;
  }
}
