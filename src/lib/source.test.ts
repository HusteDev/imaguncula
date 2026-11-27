// SPDX-License-Identifier: MIT OR Apache-2.0
import { describe, expect, it } from "vitest";

import { sourceOperationErrorMessage, sourceWarningMessage } from "./source";

describe("sourceWarningMessage", () => {
  it("renders dimension-bearing warnings", () => {
    expect(sourceWarningMessage({ code: "non-square-raster", width: 64, height: 48 })).toContain(
      "64×48",
    );
  });

  it("renders unsupported features without raw backend strings", () => {
    expect(
      sourceWarningMessage({ code: "unsupported-svg-feature-ignored", feature: "animation" }),
    ).toContain("animation");
  });
});

describe("sourceOperationErrorMessage", () => {
  it("unwraps typed preview errors", () => {
    expect(
      sourceOperationErrorMessage({ kind: "render", error: { kind: "upscale-forbidden" } }),
    ).toBe("This raster preview would upscale the original source.");
  });

  it("surfaces the exact unsafe SVG violation", () => {
    expect(
      sourceOperationErrorMessage({
        kind: "source",
        error: { kind: "unsafe-svg", violation: { code: "script-element" } },
      }),
    ).toBe("SVG script elements are forbidden.");
  });

  it("does not expose unknown object serialization", () => {
    expect(sourceOperationErrorMessage({ kind: "unknown", secret: "value" })).toBe(
      "The source operation failed.",
    );
  });
});

describe("message coverage", () => {
  it.each([
    ["embedded-image-ignored", "embedded image"],
    ["external-image-ignored", "externally linked image"],
    ["external-reference-ignored", "external SVG reference"],
    ["external-style-ignored", "external stylesheet"],
    ["font-definition-ignored", "font definition"],
    ["icc-profile-treated-as-srgb", "ICC profile"],
    ["text-ignored", "SVG text"],
  ] as const)("describes the %s warning", (code, text) => {
    expect(sourceWarningMessage({ code })).toContain(text);
  });

  it("describes a non-square viewBox", () => {
    expect(sourceWarningMessage({ code: "non-square-view-box", width: 3, height: 2 })).toContain(
      "3×2",
    );
  });

  it.each([
    [{ kind: "invalid-role" }, "role is invalid"],
    [{ kind: "file-too-large" }, "64 MiB"],
    [{ kind: "not-regular-file" }, "not a directory"],
    [{ kind: "unsupported-format" }, "content-verified"],
    [{ kind: "invalid-svg" }, "SVG is malformed"],
    [{ kind: "invalid-png" }, "PNG is malformed"],
    [{ kind: "png-decode" }, "PNG is malformed"],
    [{ kind: "png-pixel-limit" }, "safe image limits"],
    [{ kind: "png-dimension-limit" }, "safe image limits"],
    [{ kind: "png-decoded-memory-limit" }, "safe image limits"],
    [{ kind: "dimension-limit" }, "safe preview limits"],
    [{ kind: "pixel-limit" }, "safe preview limits"],
    [{ kind: "cache-unavailable" }, "cache is unavailable"],
    [{ kind: "cache-invariant" }, "cache is unavailable"],
    [{ kind: "io", message: "Disk full." }, "Disk full."],
    [{ kind: "io", message: "" }, "source operation failed"],
    [null, "source operation failed"],
    [42, "source operation failed"],
    ["Plain text.", "Plain text."],
  ])("describes the error %j", (error, text) => {
    expect(sourceOperationErrorMessage(error)).toContain(text);
  });

  it.each([
    [{ code: "document-type" }, "document type"],
    [{ code: "event-handler-attribute", attribute: "onload" }, "onload"],
    [{ code: "event-handler-attribute" }, "event-handler attributes"],
    [{ code: "foreign-object-element" }, "foreignObject"],
    [{ code: "maximum-element-count" }, "element-count"],
    [{ code: "maximum-nesting-depth" }, "nesting-depth"],
    [{ code: "script-reference" }, "script references"],
    [{ code: "unknown" }, "active or resource-amplifying"],
    [null, "active or resource-amplifying"],
  ])("describes the unsafe SVG violation %j", (violation, text) => {
    expect(sourceOperationErrorMessage({ kind: "unsafe-svg", violation })).toContain(text);
  });
});
