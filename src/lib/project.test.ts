// SPDX-License-Identifier: MIT OR Apache-2.0
import { describe, expect, it } from "vitest";

import { emptyOutputOverride, emptyProject, projectErrorMessage } from "./project";

describe("emptyProject", () => {
  it("returns the current schema and deterministic defaults", () => {
    const project = emptyProject();
    expect(project).toMatchObject({
      schemaVersion: 2,
      projectName: "Untitled project",
      baseFilename: "untitled-project",
      sources: [],
      customOutputs: [],
      outputOverrides: {},
      exportSettings: {
        createCombinedZip: true,
        overwriteConfirmed: false,
      },
      acceptedWarnings: [],
      presetMigrations: [],
    });
    expect(project.selectedPresets).toEqual([
      { id: "windows-desktop", version: 2 },
      { id: "web-pwa", version: 1 },
    ]);
    expect(project.enabledOutputs).toHaveLength(23);
    expect(project.enabledOutputs).toContain("icon-16");
    expect(project.enabledOutputs).toContain("icon-42");
    expect(project.enabledOutputs).toContain("pwa-maskable-512");
  });

  it("does not share mutable collections between projects", () => {
    const first = emptyProject();
    const second = emptyProject();

    first.enabledOutputs.push("custom-check");

    expect(second.enabledOutputs).not.toContain("custom-check");
  });
});

describe("projectErrorMessage", () => {
  it("renders structured validation issues", () => {
    expect(
      projectErrorMessage({
        kind: "validation",
        issues: [{ code: "missing-value", path: "projectName", message: "value is required" }],
      }),
    ).toBe("projectName: value is required");
  });

  it("renders unsupported schema versions", () => {
    expect(projectErrorMessage({ kind: "unsupported-schema-version", found: 9 })).toBe(
      "This project uses unsupported schema version 9.",
    );
  });
});

describe("emptyOutputOverride", () => {
  it("does not authorize an upscale or select a hidden source", () => {
    expect(emptyOutputOverride()).toEqual({
      width: null,
      height: null,
      sourceRole: null,
      sourcePath: null,
      sourceContentHash: null,
      allowUpscale: false,
    });
  });
});
