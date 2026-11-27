// SPDX-License-Identifier: MIT OR Apache-2.0
// @vitest-environment jsdom
//! Component tests for the main page. The page runs unchanged against a fake
//! desktop backend that answers Tauri commands through Tauri's IPC mock.

import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ExportReport } from "$lib/output";
import type { ProjectLoad, ProjectState } from "$lib/project";
import Page from "./+page.svelte";
import {
  installDesktop,
  sourceInfo,
  stubObjectUrls,
  uninstallDesktop,
  type FakeDesktop,
} from "../test-support/desktop";

let desktop: FakeDesktop;

beforeEach(() => {
  stubObjectUrls();
});

afterEach(() => {
  cleanup();
  uninstallDesktop();
  vi.restoreAllMocks();
});

function status(): string {
  return document.querySelector("footer .status")?.textContent ?? "";
}

async function expectStatus(text: string) {
  await waitFor(() => expect(status()).toBe(text));
}

function button(name: string | RegExp) {
  return screen.getByRole("button", { name });
}

function checkbox(name: string | RegExp) {
  return screen.getByRole("checkbox", { name }) as HTMLInputElement;
}

function confirmWith(...answers: boolean[]) {
  const confirm = vi.spyOn(window, "confirm");
  for (const answer of answers) confirm.mockReturnValueOnce(answer);
  return confirm;
}

async function renderConnected(configure?: (desktop: FakeDesktop) => void) {
  desktop = installDesktop();
  configure?.(desktop);
  const view = render(Page);
  await screen.findByText("Imaguncula desktop core connected");
  await screen.findByRole("table", { name: "Output review table" });
  await waitFor(() => expect(desktop.listening.has("export-finished")).toBe(true));
  return view;
}

async function importMaster(path = "C:\\art\\logo.svg") {
  desktop.openResults.push(path);
  await fireEvent.click(button(/Select an SVG or PNG/));
  await screen.findByRole("heading", { name: path.split(/[\\/]/).at(-1) });
  await expectStatus("Master source replaced");
}

function lastMasterPreview() {
  return desktop.calls
    .filter((call) => call.command === "render_preview" && call.args.role === "master")
    .at(-1)?.args;
}

function lastProject(): ProjectState {
  return desktop.lastArgs("resolve_outputs")?.project as ProjectState;
}

describe("desktop connection", () => {
  it("falls back to frontend preview mode without the desktop runtime", async () => {
    render(Page);
    await screen.findByText("Frontend preview mode");
    expect(screen.getByText(/Frontend preview only/)).toBeTruthy();
    await fireEvent.click(button(/Select an SVG or PNG/));
    expect(screen.getByRole("alert").textContent).toContain("requires the Tauri desktop runtime");
  });

  it("loads the starting project, presets, and outputs", async () => {
    await renderConnected();
    expect(checkbox(/^Windows desktop/).checked).toBe(true);
    expect(checkbox(/^Windows MSIX/).checked).toBe(false);
    expect(screen.getByText("2 outputs")).toBeTruthy();
    expect(screen.getByText(/Requires additional sources: foreground/)).toBeTruthy();
    expect(document.title).toBe("Untitled project — Imaguncula");
  });

  it("marks the project dirty when its identity changes", async () => {
    await renderConnected();
    const name = screen.getByLabelText("Project name");
    await fireEvent.input(name, { target: { value: "Docs" } });
    await expectStatus("Unsaved changes");
    expect(document.title).toBe("• Docs — Imaguncula");
    await fireEvent.input(screen.getByLabelText(/Base filename/), { target: { value: "docs" } });
    expect(screen.getByText("docs-icons")).toBeTruthy();
  });
});

describe("master source", () => {
  it("imports, previews, re-renders, and removes the master source", async () => {
    await renderConnected((desktop) =>
      desktop.handle("import_source", (args) =>
        sourceInfo(String(args.path), String(args.role), { warnings: [{ code: "text-ignored" }] }),
      ),
    );

    desktop.openResults.push(null);
    await fireEvent.click(button(/Select an SVG or PNG/));
    expect(desktop.lastArgs("import_source")).toBeUndefined();

    await importMaster();
    expect(screen.getByText(/SVG text is ignored/)).toBeTruthy();
    expect(screen.getByText("Preserved")).toBeTruthy();
    expect(screen.getByText("sRGB/default")).toBeTruthy();
    const image = await screen.findByAltText("Rendered preview of logo.svg");
    expect(image.getAttribute("src")).toMatch(/^blob:/);
    expect(lastMasterPreview()).toMatchObject({ width: 64, height: 64 });

    await fireEvent.change(screen.getByLabelText("Zoom"), { target: { value: "4" } });
    expect(screen.getByText(/4× inspection/)).toBeTruthy();
    expect(screen.getByAltText("Rendered preview of logo.svg").classList).toContain("pixelated");
    await fireEvent.change(screen.getByLabelText("Background"), { target: { value: "dark" } });

    await fireEvent.input(screen.getByLabelText("Width"), { target: { value: "32" } });
    await fireEvent.click(button("Render"));
    await waitFor(() => expect(lastMasterPreview()).toMatchObject({ width: 32 }));

    desktop.handle("render_preview", () => Promise.reject({ kind: "pixel-limit" }));
    await fireEvent.click(button("Render"));
    expect((await screen.findByRole("alert")).textContent).toContain("safe preview limits");
    expect(screen.getByText("Preview unavailable")).toBeTruthy();

    await fireEvent.click(button("Remove"));
    await expectStatus("Master source removed");
    expect(button(/Select an SVG or PNG/)).toBeTruthy();
    expect(lastProject().sources).toEqual([]);
  });

  it("reports import and dialog failures", async () => {
    await renderConnected();
    desktop.handle("import_source", () => Promise.reject({ kind: "unsupported-format" }));
    desktop.openResults.push("C:\\art\\notes.txt");
    await fireEvent.click(button(/Select an SVG or PNG/));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Only content-verified PNG and SVG",
    );

    desktop.handle("plugin:dialog|open", () => Promise.reject("The dialog failed."));
    await fireEvent.click(button(/Select an SVG or PNG/));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toBe("The dialog failed."));
  });

  it("imports dropped files and shows PNG facts", async () => {
    await renderConnected((desktop) =>
      desktop.handle("import_source", (args) =>
        sourceInfo(String(args.path), String(args.role), {
          png: {
            pixelCount: 4,
            bitsPerChannel: 8,
            hasAlphaChannel: true,
            hasTransparentPixels: String(args.path).includes("clear"),
            hasIccProfile: true,
          },
          dimensions: { width: 20, height: 10 },
        }),
      ),
    );

    await desktop.emit("tauri://drag-over", { position: { x: 1, y: 1 } });
    await screen.findByText("Release to inspect this source");
    await desktop.emit("tauri://drag-leave", null);
    await waitFor(() => expect(screen.queryByText("Release to inspect this source")).toBeNull());

    await desktop.emit("tauri://drag-drop", {
      paths: ["C:\\art\\opaque.png"],
      position: { x: 1, y: 1 },
    });
    await screen.findByRole("heading", { name: "opaque.png" });
    expect(screen.getByText("Opaque alpha")).toBeTruthy();
    expect(screen.getByText("ICC detected")).toBeTruthy();
    expect(lastMasterPreview()).toMatchObject({ width: 20, height: 10 });

    await desktop.emit("tauri://drag-drop", {
      paths: ["C:\\art\\clear.png"],
      position: { x: 1, y: 1 },
    });
    await screen.findByRole("heading", { name: "clear.png" });
    expect(screen.getByText("Present")).toBeTruthy();

    await desktop.emit("tauri://drag-drop", { paths: [], position: { x: 1, y: 1 } });
    expect(screen.getByRole("heading", { name: "clear.png" })).toBeTruthy();
  });

  it("describes PNG sources without an alpha channel", async () => {
    await renderConnected((desktop) =>
      desktop.handle("import_source", (args) =>
        sourceInfo(String(args.path), String(args.role), {
          png: {
            pixelCount: 4,
            bitsPerChannel: 8,
            hasAlphaChannel: false,
            hasTransparentPixels: false,
            hasIccProfile: false,
          },
        }),
      ),
    );
    await importMaster("C:\\art\\flat.png");
    expect(screen.getByText("No alpha")).toBeTruthy();
    expect(screen.getByText("No source warnings detected.")).toBeTruthy();
  });
});

describe("presets", () => {
  it("swaps exclusive presets and toggles optional groups", async () => {
    await renderConnected();

    await fireEvent.click(checkbox(/^Windows MSIX/));
    await expectStatus("Windows MSIX enabled");
    expect(checkbox(/^Windows desktop/).checked).toBe(false);
    await waitFor(() =>
      expect(lastProject().selectedPresets.map((preset) => preset.id)).toEqual(["windows-msix"]),
    );
    expect(lastProject().enabledOutputs).toEqual(["msix-44"]);
    expect(desktop.lastArgs("list_preset_upgrades")).toBeDefined();

    await fireEvent.click(checkbox(/^Windows MSIX/));
    await expectStatus("Windows MSIX disabled");
    await waitFor(() => expect(screen.getByText("No outputs selected")).toBeTruthy());

    await fireEvent.click(checkbox(/^Windows desktop/));
    const large = await screen.findByRole("checkbox", { name: /Large icon/ });
    expect(checkbox(/Package group/).disabled).toBe(true);
    await fireEvent.click(large);
    await expectStatus("Large icon enabled");
    await waitFor(() => expect(lastProject().enabledOutputs).toContain("icon-256"));
    await fireEvent.click(checkbox(/Large icon/));
    await expectStatus("Large icon disabled");
    await waitFor(() => expect(lastProject().enabledOutputs).not.toContain("icon-256"));
  });

  it("manages additional source roles for layered presets", async () => {
    await renderConnected((desktop) =>
      desktop.handle("import_source", (args) =>
        sourceInfo(String(args.path), String(args.role), {
          png: {
            pixelCount: 4,
            bitsPerChannel: 8,
            hasAlphaChannel: true,
            hasTransparentPixels: true,
            hasIccProfile: false,
          },
        }),
      ),
    );
    await fireEvent.click(checkbox(/^Android adaptive/));
    await screen.findByRole("heading", { name: "Additional source roles" });
    const foreground = screen.getByRole("heading", { name: "foreground" }).closest("article")!;
    expect(within(foreground).getByText("Required source role")).toBeTruthy();
    expect(within(foreground).getByText("No source assigned.")).toBeTruthy();
    const monochrome = screen.getByRole("heading", { name: "monochrome" }).closest("article")!;
    expect(within(monochrome).getByText("Optional source role")).toBeTruthy();
    expect(await screen.findByText("Additional sources needed")).toBeTruthy();
    expect(screen.getByText(/source still required/)).toBeTruthy();

    await fireEvent.change(screen.getByLabelText("Android launcher mask"), {
      target: { value: "circle" },
    });

    desktop.openResults.push("C:\\art\\foreground.png");
    await fireEvent.click(within(foreground).getByRole("button", { name: "Select" }));
    await expectStatus("foreground source replaced");
    expect(within(foreground).getByText("C:\\art\\foreground.png")).toBeTruthy();
    expect(within(foreground).getByText("PNG")).toBeTruthy();
    expect(within(foreground).getByText("Present")).toBeTruthy();
    await screen.findByText(/·\s*source assigned/);

    desktop.handle("plugin:dialog|open", () => Promise.reject({ kind: "not-regular-file" }));
    await fireEvent.click(within(foreground).getByRole("button", { name: "Replace" }));
    expect((await within(foreground).findByRole("alert")).textContent).toContain("not a directory");

    await fireEvent.click(within(foreground).getByRole("button", { name: "Remove" }));
    await expectStatus("foreground source removed");
    expect(lastProject().sources).toEqual([]);
  });
});

describe("preset upgrades", () => {
  const upgrade = {
    diff: {
      presetId: "windows-desktop",
      fromVersion: 2,
      toVersion: 3,
      addedOutputs: ["icon-48"],
      removedOutputs: [],
      renamedOutputs: [{ from: "icon-24", to: "icon-25" }],
    },
    diffFingerprint: "f".repeat(64),
  };

  it("reviews and accepts a preset migration", async () => {
    await renderConnected((desktop) => desktop.handle("list_preset_upgrades", () => [upgrade]));
    await screen.findByRole("heading", { name: "Preset updates" });
    expect(screen.getByText("Version 2 → 3")).toBeTruthy();
    expect(screen.getByText("icon-48")).toBeTruthy();
    expect(screen.getByText("icon-25")).toBeTruthy();
    expect(screen.getByText("None")).toBeTruthy();

    desktop.handle("apply_preset_upgrade", (args) => ({
      ...(args.project as ProjectState),
      selectedPresets: [{ id: "windows-desktop", version: 3 }],
    }));
    desktop.handle("list_preset_upgrades", () => []);
    await fireEvent.click(button("Accept and migrate"));
    await expectStatus("windows-desktop migrated to version 3");
    expect(desktop.lastArgs("apply_preset_upgrade")).toMatchObject({
      presetId: "windows-desktop",
      targetVersion: 3,
      acceptedDiffFingerprint: upgrade.diffFingerprint,
    });
    await waitFor(() =>
      expect(screen.queryByRole("heading", { name: "Preset updates" })).toBeNull(),
    );
  });

  it("reports a rejected migration and survives a failed upgrade listing", async () => {
    await renderConnected((desktop) => desktop.handle("list_preset_upgrades", () => [upgrade]));
    desktop.handle("apply_preset_upgrade", () => Promise.reject({ kind: "validation" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Accept and migrate" }));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "The project contains invalid values.",
    );

    desktop.handle("list_preset_upgrades", () => Promise.reject(new Error("unavailable")));
    await fireEvent.click(checkbox(/^Windows MSIX/));
    await waitFor(() =>
      expect(screen.queryByRole("heading", { name: "Preset updates" })).toBeNull(),
    );
  });
});

describe("custom outputs", () => {
  async function addFirstCustomOutput() {
    await fireEvent.click(checkbox(/^Windows desktop/));
    await fireEvent.click(await screen.findByRole("button", { name: "Add first output" }));
    await expectStatus("Custom output added");
    return screen.findByRole("textbox", { name: "Filename" });
  }

  it("adds, edits, includes, and removes a custom output", async () => {
    await renderConnected();
    await importMaster();
    const filename = await addFirstCustomOutput();
    expect(lastProject().customOutputs).toHaveLength(1);

    await fireEvent.change(filename, { target: { value: " " } });
    expect(screen.getByRole("alert").textContent).toBe("Filename cannot be empty.");
    await fireEvent.change(screen.getByRole("textbox", { name: "Usage" }), {
      target: { value: "" },
    });
    expect(screen.getByRole("alert").textContent).toBe("Usage cannot be empty.");

    await fireEvent.change(filename, { target: { value: "app.png" } });
    await waitFor(() => expect(lastProject().customOutputs[0].filename).toBe("app.png"));
    await fireEvent.change(screen.getByRole("textbox", { name: "Destination" }), {
      target: { value: "store" },
    });
    await waitFor(() => expect(lastProject().customOutputs[0].destination).toBe("store"));
    await fireEvent.change(screen.getByRole("textbox", { name: "Usage" }), {
      target: { value: "Store listing" },
    });
    const [width, height] = screen.getAllByRole("spinbutton").slice(-2);
    await fireEvent.change(width, { target: { value: "99999" } });
    await waitFor(() => expect(lastProject().customOutputs[0].width).toBe(16_384));
    await fireEvent.change(height, { target: { value: "abc" } });
    await waitFor(() => expect(lastProject().customOutputs[0].height).toBe(1));

    await fireEvent.click(checkbox("Include app.png"));
    await waitFor(() => expect(lastProject().enabledOutputs).not.toContain("custom-1"));
    await fireEvent.click(checkbox("Include app.png"));
    await waitFor(() => expect(lastProject().enabledOutputs).toContain("custom-1"));

    await fireEvent.click(button("Add custom output"));
    await waitFor(() => expect(lastProject().customOutputs).toHaveLength(2));

    await fireEvent.click(screen.getAllByRole("button", { name: "Remove" }).at(-1)!);
    await expectStatus("Custom output removed");
    await waitFor(() => expect(lastProject().customOutputs).toHaveLength(1));
  });

  it("replaces, resets, and upscales an output source", async () => {
    await renderConnected();
    await importMaster();
    await addFirstCustomOutput();

    desktop.openResults.push(null);
    await fireEvent.click(button("Replace source"));
    expect(desktop.calls.filter((call) => call.args.role === "override")).toEqual([]);

    desktop.openResults.push("C:\\art\\small.png");
    await fireEvent.click(button("Replace source"));
    await expectStatus("Source replaced for custom-1");
    await screen.findByText("Per-output override");

    const [width] = screen.getAllByRole("spinbutton").slice(-2);
    await fireEvent.change(width, { target: { value: "512" } });
    const upscale = await screen.findByRole("checkbox", { name: "Allow upscale" });
    expect(screen.getByText("The raster source would be upscaled.")).toBeTruthy();
    await fireEvent.click(upscale);
    await waitFor(() => expect(lastProject().outputOverrides["custom-1"].allowUpscale).toBe(true));

    await fireEvent.click(button("Reset source"));
    await expectStatus("Source reset for custom-1");
    await waitFor(() => expect(lastProject().outputOverrides["custom-1"].sourcePath).toBeNull());

    desktop.handle("import_source", () => Promise.reject({ kind: "file-too-large" }));
    desktop.openResults.push("C:\\art\\huge.png");
    await fireEvent.click(button("Replace source"));
    expect((await screen.findByRole("alert")).textContent).toContain("64 MiB");
  });

  it("stores dimension overrides for preset outputs and filters by platform", async () => {
    await renderConnected();
    await fireEvent.click(checkbox(/^Android adaptive/));
    await screen.findByText("3 outputs");

    const [width] = screen.getAllByRole("spinbutton");
    await fireEvent.change(width, { target: { value: "20" } });
    await waitFor(() => expect(lastProject().outputOverrides["icon-16"].width).toBe(20));

    await fireEvent.change(screen.getByLabelText("Platform filter"), {
      target: { value: "android" },
    });
    expect(screen.getByText("1 output")).toBeTruthy();
    expect(screen.getByText("Additional role: foreground")).toBeTruthy();
  });

  it("reports output resolution failures", async () => {
    await renderConnected();
    desktop.handle("resolve_outputs", () => Promise.reject({ kind: "outputs-not-ready" }));
    await fireEvent.click(checkbox(/^Windows MSIX/));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "Every included output must be ready before export.",
    );
  });
});

describe("export", () => {
  const report: ExportReport = {
    destination: "C:\\out\\untitled-project-icons",
    reportMarkdown: "# Report",
    report: {
      projectName: "Untitled project",
      baseFilename: "untitled-project",
      files: [
        {
          outputId: "icon-16",
          path: "windows/icon-16.png",
          width: 16,
          height: 16,
          format: "png",
          sourceLabel: "master",
          sourcePath: "C:\\art\\logo.svg",
          sourceContentHash: "c".repeat(64),
        },
      ],
      warnings: [
        { outputId: "icon-16", code: "small", message: "Very small output.", accepted: true },
        { outputId: "icon-32", code: "thin", message: "Thin strokes.", accepted: false },
      ],
    },
  };

  function inspection(state: string, requiresConfirmation = false) {
    return () => ({ path: "C:\\out\\untitled-project-icons", state, requiresConfirmation });
  }

  it("refuses to export outputs that have errors", async () => {
    await renderConnected();
    await fireEvent.click(button("Create"));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "Resolve every included output error before creating the package.",
    );
  });

  it("stops when the destination is cancelled, foreign, or not confirmed", async () => {
    await renderConnected();
    await importMaster();

    desktop.openResults.push(null);
    await fireEvent.click(button("Create"));
    await expectStatus("Export cancelled");

    desktop.handle("inspect_export_destination", inspection("non-empty-directory"));
    desktop.openResults.push("C:\\out");
    await fireEvent.click(button("Create"));
    expect((await screen.findByRole("alert")).textContent).toContain("it is never replaced");

    desktop.handle("inspect_export_destination", inspection("non-empty-package-directory", true));
    confirmWith(false);
    desktop.openResults.push("C:\\out");
    await fireEvent.click(button("Create"));
    await expectStatus("Export cancelled");
    expect(desktop.lastArgs("start_export")).toBeUndefined();
  });

  it("runs an export, shows progress, cancels, and renders the report", async () => {
    await renderConnected();
    await importMaster();
    desktop.handle("inspect_export_destination", inspection("non-empty-package-directory", true));
    desktop.handle("start_export", () => "export-7");
    desktop.handle("cancel_export", () => true);
    confirmWith(true);
    desktop.openResults.push("C:\\out");

    await fireEvent.click(button("Create"));
    await expectStatus("Creating output package");
    expect(desktop.lastArgs("start_export")).toMatchObject({ parent: "C:\\out", overwrite: true });
    expect(screen.getByText(/0 of 2/).textContent).toContain("preparing");

    await desktop.emit("export-progress", {
      jobId: "export-7",
      completed: 1,
      total: 2,
      currentOutput: "icon-16",
    });
    await screen.findByText(/1 of 2\s+· icon-16/);
    await desktop.emit("export-progress", {
      jobId: "other",
      completed: 2,
      total: 2,
      currentOutput: null,
    });
    expect(screen.getByText(/1 of 2/)).toBeTruthy();

    await fireEvent.click(button("Cancel export"));
    await expectStatus("Cancelling export");
    expect(desktop.lastArgs("cancel_export")).toEqual({ jobId: "export-7" });

    await desktop.emit("export-finished", { jobId: "other", report: null, error: "ignored" });
    expect(button("Cancel export")).toBeTruthy();

    await desktop.emit("export-finished", { jobId: "export-7", report, error: null });
    await expectStatus("Created 1 output file(s)");
    expect(screen.getByRole("heading", { name: "Generation report" })).toBeTruthy();
    expect(screen.getByText("windows/icon-16.png")).toBeTruthy();
    expect(screen.getByText("16×16")).toBeTruthy();
    expect(screen.getByText("Very small output.").closest("li")?.textContent).toContain("Accepted");
    expect(screen.getByText("Thin strokes.").closest("li")?.textContent).toContain("Outstanding");
    expect(button("Create")).toBeTruthy();
  });

  it("handles events that arrive before the start command returns", async () => {
    await renderConnected();
    await importMaster();
    desktop.handle("inspect_export_destination", inspection("missing"));
    desktop.handle("start_export", async () => {
      await desktop.emit("export-progress", {
        jobId: "export-9",
        completed: 0,
        total: 2,
        currentOutput: null,
      });
      await desktop.emit("export-finished", {
        jobId: "export-9",
        report: { ...report, report: { ...report.report, files: [], warnings: [] } },
        error: null,
      });
      return "export-9";
    });
    desktop.openResults.push("C:\\out");
    await fireEvent.click(button("Create"));
    await expectStatus("Created 0 output file(s)");
    expect(screen.getByText("No generated files.")).toBeTruthy();
    expect(screen.getByText("No warnings.")).toBeTruthy();
    expect(button("Create")).toBeTruthy();
  });

  it("reports failed exports, start failures, and cancel failures", async () => {
    await renderConnected();
    await importMaster();
    desktop.handle("inspect_export_destination", inspection("empty-directory"));
    desktop.handle("start_export", () => "export-1");
    desktop.handle("cancel_export", () => Promise.reject({ kind: "jobs-unavailable" }));
    desktop.openResults.push("C:\\out");
    await fireEvent.click(button("Create"));
    await expectStatus("Creating output package");

    await fireEvent.click(button("Cancel export"));
    expect((await screen.findByRole("alert")).textContent).toContain("export job manager");

    await desktop.emit("export-finished", {
      jobId: "export-1",
      report: null,
      error: { kind: "cancelled" },
    });
    await expectStatus("Export failed");
    expect(screen.getByRole("alert").textContent).toContain("No incomplete package");

    desktop.handle("start_export", () => Promise.reject({ kind: "job-already-running" }));
    desktop.openResults.push("C:\\out");
    await fireEvent.click(button("Create"));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toBe("Another export is already running."),
    );
  });

  it("turns off the combined ZIP", async () => {
    await renderConnected();
    await fireEvent.click(checkbox("Include combined ZIP"));
    await expectStatus("Unsaved changes");
    expect(checkbox("Include combined ZIP").checked).toBe(false);
  });
});

describe("project files", () => {
  function loaded(overrides: Partial<ProjectLoad> = {}): ProjectLoad {
    return {
      project: {
        schemaVersion: 2,
        projectName: "Saved",
        baseFilename: "saved",
        sources: [
          { role: "master", path: "C:\\proj\\logo.svg", contentHash: "a".repeat(64) },
          { role: "foreground", path: "C:\\elsewhere\\fg.png", contentHash: "e".repeat(64) },
        ],
        selectedPresets: [{ id: "android-adaptive", version: 1 }],
        enabledOutputs: ["adaptive-foreground"],
        customOutputs: [],
        outputOverrides: {
          "adaptive-foreground": {
            width: null,
            height: null,
            sourceRole: null,
            sourcePath: "C:\\elsewhere\\override.png",
            sourceContentHash: "d".repeat(64),
            allowUpscale: true,
          },
        },
        exportSettings: { createCombinedZip: true, overwriteConfirmed: false },
        acceptedWarnings: [{ code: "small", outputId: null }],
        presetMigrations: [],
      },
      migrationNotices: ["Upgraded schema"],
      externalSources: [
        { reference: "sources.foreground", path: "C:\\elsewhere\\fg.png" },
        { reference: "overrides.adaptive-foreground", path: "C:\\elsewhere\\override.png" },
      ],
      ...overrides,
    };
  }

  it("saves through the save dialog and then in place", async () => {
    await renderConnected();
    desktop.saveResults.push(null);
    await fireEvent.click(button("Save"));
    await expectStatus("Save cancelled");

    desktop.saveResults.push("C:\\proj\\icons");
    await fireEvent.click(button("Save"));
    await expectStatus("Project saved");
    expect(desktop.lastArgs("save_project")).toMatchObject({
      path: "C:\\proj\\icons.imaguncula.json",
    });
    expect(screen.getByText("C:\\proj\\icons.imaguncula.json")).toBeTruthy();

    await fireEvent.click(button("Save"));
    await expectStatus("Project saved");
    expect(desktop.calls.filter((call) => call.command === "plugin:dialog|save")).toHaveLength(2);
  });

  it("normalizes project file extensions and reports save errors", async () => {
    await renderConnected();
    desktop.saveResults.push("C:\\proj\\a.json");
    await fireEvent.click(button("Save"));
    await expectStatus("Project saved");
    expect(desktop.lastArgs("save_project")?.path).toBe("C:\\proj\\a.imaguncula.json");

    await fireEvent.click(button("New"));
    await expectStatus("New unsaved project");
    desktop.saveResults.push("C:\\proj\\b.IMAGUNCULA.JSON");
    desktop.handle("save_project", () => Promise.reject({ kind: "invalid-file-name" }));
    await fireEvent.click(button("Save"));
    expect((await screen.findByRole("alert")).textContent).toContain(".imaguncula.json extension");
    expect(desktop.lastArgs("save_project")?.path).toBe("C:\\proj\\b.IMAGUNCULA.JSON");
  });

  it("asks before discarding unsaved changes", async () => {
    await renderConnected();
    await fireEvent.input(screen.getByLabelText("Project name"), { target: { value: "Draft" } });
    const confirm = confirmWith(false, false, true);

    await fireEvent.click(button("New"));
    await fireEvent.click(button("Open"));
    expect(confirm).toHaveBeenCalledTimes(2);
    expect(status()).toBe("Unsaved changes");

    desktop.handle("new_project", () => Promise.reject("The core is unavailable."));
    await fireEvent.click(button("New"));
    expect((await screen.findByRole("alert")).textContent).toBe("The core is unavailable.");
  });

  it("opens a project, drops untrusted external sources, and reports notices", async () => {
    await renderConnected();
    desktop.handle("load_project", () => loaded());
    desktop.handle("import_source", (args) =>
      String(args.role) === "foreground"
        ? Promise.reject({ kind: "invalid-png" })
        : sourceInfo(String(args.path), String(args.role)),
    );

    desktop.openResults.push(null);
    await fireEvent.click(button("Open"));
    await expectStatus("Open cancelled");

    confirmWith(false);
    desktop.openResults.push("C:\\proj\\saved.imaguncula.json");
    await fireEvent.click(button("Open"));
    await expectStatus(
      "Project opened with 1 migration notice(s), 1 pre-accepted warning(s), 1 pre-authorized upscale(s)",
    );
    expect(screen.getByText("C:\\proj\\saved.imaguncula.json")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "logo.svg" })).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toContain(
      "source contents changed since this project was saved",
    );
    expect(lastProject().sources.map((source) => source.role)).toEqual(["master"]);
    expect(lastProject().outputOverrides["adaptive-foreground"].sourcePath).toBeNull();
    expect(document.title).toBe("• Saved — Imaguncula");
  });

  it("keeps trusted external sources and inspects each role", async () => {
    await renderConnected();
    desktop.handle("load_project", () => loaded({ migrationNotices: [] }));
    confirmWith(true);
    desktop.openResults.push("C:\\proj\\saved.imaguncula.json");
    await fireEvent.click(button("Open"));
    await waitFor(() => expect(status()).toMatch(/^Project opened with 1 pre-accepted/));
    const foreground = screen.getByRole("heading", { name: "foreground" }).closest("article")!;
    expect(within(foreground).getByText("C:\\elsewhere\\fg.png")).toBeTruthy();
    expect((await within(foreground).findByRole("alert")).textContent).toContain(
      "source contents changed",
    );
  });

  it("opens a plain project and reports load failures", async () => {
    await renderConnected();
    const plain = loaded({ externalSources: [], migrationNotices: [] });
    plain.project.sources = [];
    plain.project.acceptedWarnings = [];
    plain.project.outputOverrides = {};
    desktop.handle("load_project", () => plain);
    desktop.openResults.push("C:\\proj\\plain.imaguncula.json");
    await fireEvent.click(button("Open"));
    await expectStatus("Project opened");
    expect(document.title).toBe("Saved — Imaguncula");

    desktop.handle("load_project", () =>
      Promise.reject({
        kind: "validation",
        issues: [{ code: "x", path: "sources[0]", message: "bad path" }],
      }),
    );
    desktop.openResults.push("C:\\proj\\broken.imaguncula.json");
    await fireEvent.click(button("Open"));
    expect((await screen.findByRole("alert")).textContent).toBe("sources[0]: bad path");
  });
});
