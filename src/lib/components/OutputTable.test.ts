// SPDX-License-Identifier: MIT OR Apache-2.0
// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { OutputSpec } from "$lib/output";
import OutputTable from "./OutputTable.svelte";
import { installDesktop, stubObjectUrls, uninstallDesktop } from "../../test-support/desktop";

let urls: ReturnType<typeof stubObjectUrls>;

beforeEach(() => {
  urls = stubObjectUrls();
});

afterEach(() => {
  cleanup();
  uninstallDesktop();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

function output(id: string, overrides: Partial<OutputSpec> = {}): OutputSpec {
  return {
    id,
    presetId: "android-adaptive",
    presetVersion: 1,
    included: true,
    platform: "android",
    usage: `${id} usage`,
    width: 48,
    height: 48,
    maximumBytes: null,
    format: "png",
    filename: `${id}.png`,
    destination: "android",
    packagePath: `android/${id}.png`,
    packageId: "files",
    packageKind: "file",
    sourceRole: "master",
    sourceLabel: "master",
    sourcePath: "C:\\art\\logo.svg",
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

function props(outputs: OutputSpec[]) {
  return {
    outputs,
    onToggle: vi.fn(),
    onUpdate: vi.fn(),
    onRemove: vi.fn(),
    onReplace: vi.fn(),
    onResetSource: vi.fn(),
    onAllowUpscale: vi.fn(),
  };
}

describe("OutputTable", () => {
  it("renders previews with launcher masks and the safe zone", async () => {
    installDesktop();
    render(OutputTable, props([output("maskable", { safeZonePercent: 80 })]));
    const image = await screen.findByAltText("Preview of maskable.png at 48 by 48 pixels");
    expect(screen.getByText("80% safe-zone circle shown")).toBeTruthy();
    expect(image.classList).toContain("pixel-aligned");

    const mask = screen.getByLabelText("Android launcher mask");
    for (const [value, className] of [
      ["circle", "mask-circle"],
      ["squircle", "mask-squircle"],
      ["rounded-square", "mask-rounded-square"],
    ]) {
      await fireEvent.change(mask, { target: { value } });
      expect(image.classList).toContain(className);
    }
  });

  it("shows why a preview is unavailable", async () => {
    const desktop = installDesktop();
    desktop.handle("render_preview", () => Promise.reject({ kind: "upscale-forbidden" }));
    render(
      OutputTable,
      props([output("missing", { sourcePath: null, sourceContentHash: null }), output("failing")]),
    );
    expect(await screen.findByText("Source required")).toBeTruthy();
    expect(await screen.findByText(/would upscale the original source/)).toBeTruthy();
  });

  it("times out previews that never finish", async () => {
    vi.useFakeTimers();
    const desktop = installDesktop();
    desktop.handle("render_preview", () => new Promise(() => {}));
    render(OutputTable, props([output("slow")]));
    await vi.advanceTimersByTimeAsync(15_000);
    vi.useRealTimers();
    expect(await screen.findByText(/Preview rendering timed out/)).toBeTruthy();
  });

  it("re-renders previews when the source changes and discards stale results", async () => {
    const desktop = installDesktop();
    const pending: Array<() => void> = [];
    desktop.handle(
      "render_preview",
      (args) =>
        new Promise((resolve) =>
          pending.push(() =>
            resolve({
              contentHash: "a".repeat(64),
              width: args.width,
              height: args.height,
              pngBytes: [1],
              cacheHit: false,
            }),
          ),
        ),
    );
    const view = render(OutputTable, props([output("icon")]));
    await waitFor(() => expect(pending).toHaveLength(1));

    await view.rerender(props([output("icon", { sourceContentHash: "b".repeat(64) })]));
    await waitFor(() => expect(pending).toHaveLength(2));
    pending[0]();
    await waitFor(() => expect(urls.revoke).toHaveBeenCalledWith("blob:preview-1"));
    expect(screen.queryByRole("img")).toBeNull();

    pending[1]();
    await screen.findByAltText(/Preview of icon.png/);

    await view.rerender(
      props([output("icon", { allowUpscale: true, sourceContentHash: "c".repeat(64) })]),
    );
    await waitFor(() => expect(pending).toHaveLength(3));
    pending[2]();
    await waitFor(() => expect(urls.revoke).toHaveBeenCalledWith("blob:preview-2"));

    await view.rerender(props([output("icon", { sourcePath: null, sourceContentHash: null })]));
    expect(await screen.findByText("Source required")).toBeTruthy();
  });

  it("edits custom rows and forwards row actions", async () => {
    installDesktop();
    const handlers = props([
      output("custom-1", {
        presetId: null,
        presetVersion: null,
        platform: "custom",
        sourceRole: "foreground",
        sourceLabel: "foreground",
        sourceOverride: true,
        allowUpscale: true,
        status: "warning",
        issues: [{ code: "thin", severity: "warning", message: "Thin strokes." }],
      }),
      output("vector", {
        presetId: null,
        presetVersion: null,
        platform: "custom",
        format: "svg",
        width: null,
        height: null,
        included: false,
      }),
    ]);
    render(OutputTable, handlers);

    expect(screen.getByText("Scalable")).toBeTruthy();
    expect(screen.getByText("Thin strokes.")).toBeTruthy();
    expect(screen.getByText("Additional role: foreground")).toBeTruthy();
    expect(screen.getByText("Additional sources needed")).toBeTruthy();
    expect(screen.getByText("No")).toBeTruthy();

    await fireEvent.click(screen.getByRole("checkbox", { name: "Allow upscale" }));
    expect(handlers.onAllowUpscale).toHaveBeenCalledWith("custom-1", false);
    await fireEvent.click(screen.getAllByRole("button", { name: "Replace source" })[0]);
    expect(handlers.onReplace).toHaveBeenCalledWith("custom-1");
    await fireEvent.click(screen.getByRole("button", { name: "Reset source" }));
    expect(handlers.onResetSource).toHaveBeenCalledWith("custom-1");
    await fireEvent.click(screen.getByRole("checkbox", { name: "Include vector.png" }));
    expect(handlers.onToggle).toHaveBeenCalledWith("vector", true);

    await screen.findAllByRole("img");
    await fireEvent.click(screen.getAllByRole("button", { name: "Remove" })[0]);
    expect(handlers.onRemove).toHaveBeenCalledWith("custom-1");
    expect(urls.revoke).toHaveBeenCalled();
  });

  it("scrolls new rows into view and releases previews when destroyed", async () => {
    installDesktop();
    const outputs = Array.from({ length: 12 }, (_, index) =>
      output(`icon-${index}`, { platform: index % 2 ? "android" : "web" }),
    );
    const view = render(OutputTable, props(outputs));
    await screen.findAllByRole("img");

    view.component.scrollToOutput("icon-11");
    view.component.scrollToOutput("missing");
    await waitFor(() => expect(screen.getByAltText(/Preview of icon-11.png/)).toBeTruthy());

    await fireEvent.scroll(document.querySelector(".table-viewport")!, {
      target: { scrollTop: 0 },
    });
    await fireEvent.change(screen.getByLabelText("Platform filter"), { target: { value: "web" } });
    expect(screen.getByText("6 outputs")).toBeTruthy();

    const created = urls.create.mock.calls.length;
    view.unmount();
    expect(urls.revoke.mock.calls.length).toBeGreaterThanOrEqual(created);
  });
});
