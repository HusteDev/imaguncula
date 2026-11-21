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
