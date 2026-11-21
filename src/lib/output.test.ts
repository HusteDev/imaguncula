// SPDX-License-Identifier: MIT OR Apache-2.0
import { describe, expect, it } from "vitest";

import {
  additionalSourceRoleDependencies,
  additionalSourceRoleNotice,
  additionalSourceRoleRequirements,
  exportErrorMessage,
  nextCustomOutput,
  outputPreviewLayout,
  reportDimensions,
} from "./output";
import type { GeneratedFile, OutputSpec } from "./output";
import type { CustomOutput, PresetSummary } from "./project";

function outputSpec(overrides: Partial<OutputSpec>): OutputSpec {
  return {
    id: "output",
    presetId: "preset",
    presetVersion: 1,
    included: true,
    platform: "test",
    usage: "Test output",
    width: 64,
    height: 64,
    maximumBytes: null,
    format: "png",
    filename: "output.png",
    destination: "icons",
    packagePath: "icons/output.png",
    packageId: "files",
    packageKind: "file",
    sourceRole: "master",
    sourceLabel: "master",
    sourcePath: "master.svg",
    sourceContentHash: "a".repeat(64),
    sourceOverride: false,
    allowUpscale: false,
    flattenColor: null,
    safeZonePercent: null,
    status: "ready",
    issues: [],
    ...overrides,
  };
}

function generatedFile(overrides: Partial<GeneratedFile>): GeneratedFile {
  return {
    outputId: "output",
    path: "icons/output.png",
    width: 64,
    height: 64,
    format: "png",
    sourceLabel: "master",
    sourcePath: "master.svg",
    sourceContentHash: "a".repeat(64),
    ...overrides,
  };
}

function presetSummary(overrides: Partial<PresetSummary>): PresetSummary {
  return {
    id: "preset",
    version: 1,
    displayName: "Preset output",
    exclusiveGroup: null,
    platform: "custom",
    sourceRoles: [{ role: "master", required: true, description: "Master artwork" }],
    outputIds: ["output"],
    defaultOutputIds: ["output"],
    optionalOutputGroups: [],
    ...overrides,
  };
}

describe("nextCustomOutput", () => {
  it("creates a unique PNG row with deterministic defaults", () => {
    const existing: CustomOutput[] = [
      {
        id: "custom-2",
        width: 16,
        height: 16,
        format: "png",
        filename: "existing.png",
        destination: "custom",
        sourceRole: "master",
        usage: "Existing",
      },
    ];

    expect(nextCustomOutput(existing, "sample-app")).toEqual({
      id: "custom-3",
      width: 64,
      height: 64,
      format: "png",
      filename: "sample-app-3.png",
      destination: "custom",
      sourceRole: "master",
      usage: "Custom application icon",
    });
  });
});

describe("outputPreviewLayout", () => {
  it("displays Windows icon sizes without fractional enlargement", () => {
    expect(outputPreviewLayout(40, 40)).toEqual({
      renderWidth: 40,
      renderHeight: 40,
      displayWidth: 40,
      displayHeight: 40,
      integerZoom: 1,
    });
    expect(outputPreviewLayout(42, 42)).toEqual({
      renderWidth: 42,
      renderHeight: 42,
      displayWidth: 42,
      displayHeight: 42,
      integerZoom: 1,
    });
    expect(outputPreviewLayout(48, 48)).toEqual({
      renderWidth: 48,
      renderHeight: 48,
      displayWidth: 48,
      displayHeight: 48,
      integerZoom: 1,
    });
  });

  it("uses the largest integer zoom that fits for small previews", () => {
    expect(outputPreviewLayout(16, 16)).toMatchObject({
      displayWidth: 64,
      displayHeight: 64,
      integerZoom: 4,
    });
    expect(outputPreviewLayout(32, 16)).toMatchObject({
      displayWidth: 64,
      displayHeight: 32,
      integerZoom: 2,
    });
  });

  it("fits large previews proportionally after capping render work", () => {
    expect(outputPreviewLayout(512, 256)).toEqual({
      renderWidth: 256,
      renderHeight: 128,
      displayWidth: 64,
      displayHeight: 32,
      integerZoom: null,
    });
  });
});

describe("additionalSourceRoleRequirements", () => {
  it("groups included outputs that need non-master artwork and reports missing roles", () => {
    const requirements = additionalSourceRoleRequirements([
      outputSpec({ id: "master", sourceRole: "master" }),
      outputSpec({ id: "foreground-1", sourceRole: "foreground" }),
      outputSpec({ id: "foreground-2", sourceRole: "foreground" }),
      outputSpec({
        id: "background",
        sourceRole: "background",
        sourcePath: null,
        sourceContentHash: null,
        status: "error",
      }),
      outputSpec({ id: "excluded", sourceRole: "monochrome", included: false }),
      outputSpec({ id: "override", sourceRole: "override", sourceOverride: true }),
    ]);

    expect(requirements).toEqual([
      { role: "background", outputIds: ["background"], configured: false },
      {
        role: "foreground",
        outputIds: ["foreground-1", "foreground-2"],
        configured: true,
      },
    ]);
  });
});

describe("additional preset source roles", () => {
  it("writes an explicit notice for output types that need more than the master source", () => {
    expect(
      additionalSourceRoleNotice(
        presetSummary({
          sourceRoles: [
            { role: "master", required: true, description: "Master artwork" },
            { role: "foreground", required: true, description: "Foreground artwork" },
            { role: "background", required: true, description: "Background artwork" },
            { role: "monochrome", required: false, description: "Themed artwork" },
          ],
        }),
      ),
    ).toBe("Requires additional sources: foreground and background. Optional source: monochrome.");
    expect(additionalSourceRoleNotice(presetSummary({}))).toBeNull();
  });

  it("maps each additional role back to the selected output types it supports", () => {
    const dependencies = additionalSourceRoleDependencies(
      [
        presetSummary({
          id: "android",
          displayName: "Android launcher",
          sourceRoles: [
            { role: "master", required: true, description: "Master artwork" },
            { role: "foreground", required: true, description: "Foreground artwork" },
            { role: "monochrome", required: false, description: "Themed artwork" },
          ],
        }),
        presetSummary({
          id: "composer",
          displayName: "Apple Icon Composer",
          sourceRoles: [{ role: "foreground", required: true, description: "Foreground layer" }],
        }),
        presetSummary({
          id: "unselected",
          displayName: "Unselected output",
          sourceRoles: [{ role: "foreground", required: true, description: "Unused" }],
        }),
      ],
      ["android", "composer"],
    );

    expect(dependencies).toEqual([
      {
        role: "foreground",
        required: true,
        description: "Foreground artwork",
        outputTypes: [
          { presetId: "android", displayName: "Android launcher", required: true },
          { presetId: "composer", displayName: "Apple Icon Composer", required: true },
        ],
      },
      {
        role: "monochrome",
        required: false,
        description: "Themed artwork",
        outputTypes: [{ presetId: "android", displayName: "Android launcher", required: false }],
      },
    ]);
  });
});

describe("reportDimensions", () => {
  it("formats fixed, multi-size, scalable, and non-dimensional report rows", () => {
    expect(reportDimensions(generatedFile({ width: 32, height: 48 }))).toBe("32×48");
    expect(reportDimensions(generatedFile({ width: null, height: null, format: "ico" }))).toBe(
      "Multi-size",
    );
    expect(reportDimensions(generatedFile({ width: null, height: null, format: "svg" }))).toBe(
      "Scalable",
    );
    expect(reportDimensions(generatedFile({ width: null, height: null, format: "json" }))).toBe(
      "N/A",
    );
  });
});

describe("exportErrorMessage", () => {
  it("unwraps nested typed core errors", () => {
    expect(exportErrorMessage({ kind: "resolve", error: { kind: "outputs-not-ready" } })).toBe(
      "Every included output must be ready before export.",
    );
  });

  it("explains cancellation without implying partial output", () => {
    expect(exportErrorMessage({ kind: "cancelled" })).toContain("No incomplete package");
  });

  it("explains disabled metadata references", () => {
    expect(exportErrorMessage({ kind: "metadata-reference-missing" })).toContain(
      "Re-enable that output",
    );
  });
});
