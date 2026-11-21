<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, untrack } from "svelte";

  import {
    additionalSourceRoleRequirements,
    type CustomOutputField,
    type OutputSpec,
    outputPreviewLayout,
  } from "$lib/output";
  import { sourceOperationErrorMessage, type PreviewImage } from "$lib/source";

  interface Props {
    outputs: OutputSpec[];
    onToggle: (id: string, included: boolean) => void;
    onUpdate: (id: string, field: CustomOutputField, value: string | number) => void;
    onRemove: (id: string) => void;
    onReplace: (id: string) => void;
    onResetSource: (id: string) => void;
    onAllowUpscale: (id: string, allowed: boolean) => void;
  }

  interface PreviewEntry {
    signature: string;
    url: string;
    error: string;
  }

  let { outputs, onToggle, onUpdate, onRemove, onReplace, onResetSource, onAllowUpscale }: Props =
    $props();

  const ROW_HEIGHT = 176;
  const VIEWPORT_HEIGHT = 560;
  const OVERSCAN = 3;
  const PREVIEW_TIMEOUT_MS = 15_000;

  let platformFilter = $state("all");
  let launcherShape = $state<"none" | "circle" | "squircle" | "rounded-square">("none");
  let scrollTop = $state(0);
  let viewportElement = $state<HTMLDivElement | null>(null);
  let previews = $state<Record<string, PreviewEntry>>({});
  let filteredOutputs = $derived(
    platformFilter === "all"
      ? outputs
      : outputs.filter((output) => output.platform === platformFilter),
  );
  let maximumStartIndex = $derived(
    Math.max(0, filteredOutputs.length - Math.ceil(VIEWPORT_HEIGHT / ROW_HEIGHT)),
  );
  let startIndex = $derived(
    Math.min(maximumStartIndex, Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN)),
  );
  let endIndex = $derived(
    Math.min(
      filteredOutputs.length,
      Math.ceil((scrollTop + VIEWPORT_HEIGHT) / ROW_HEIGHT) + OVERSCAN,
    ),
  );
  let visibleOutputs = $derived(filteredOutputs.slice(startIndex, endIndex));
  let platforms = $derived([...new Set(outputs.map((output) => output.platform))].sort());
  let additionalRoles = $derived(additionalSourceRoleRequirements(outputs));

  /**
   * Scrolls the virtualized viewport so the identified row is visible.
   *
   * Rows outside the viewport are not rendered at all, so callers that add a
   * row must bring it into view or keyboard and assistive-technology users
   * never encounter it.
   */
  export function scrollToOutput(id: string) {
    const index = filteredOutputs.findIndex((output) => output.id === id);
    if (index < 0 || !viewportElement) return;
    const maximumScrollTop = Math.max(0, filteredOutputs.length * ROW_HEIGHT - VIEWPORT_HEIGHT);
    viewportElement.scrollTop = Math.min(index * ROW_HEIGHT, maximumScrollTop);
    scrollTop = viewportElement.scrollTop;
  }

  $effect(() => {
    const outputsToPreview = visibleOutputs;
    untrack(() => {
      for (const output of outputsToPreview) {
        void ensurePreview(output);
      }
    });
  });

  onDestroy(() => {
    for (const preview of Object.values(previews)) {
      if (preview.url) URL.revokeObjectURL(preview.url);
    }
  });

  async function ensurePreview(output: OutputSpec) {
    if (!output.sourcePath || !output.sourceContentHash) {
      const signature = "source-unavailable";
      if (previews[output.id]?.signature === signature) return;
      if (previews[output.id]?.url) URL.revokeObjectURL(previews[output.id].url);
      previews = {
        ...previews,
        [output.id]: {
          signature,
          url: "",
          error: "Source required",
        },
      };
      return;
    }
    const { renderWidth: width, renderHeight: height } = outputPreviewLayout(
      output.width,
      output.height,
    );
    const signature = [
      output.sourceContentHash,
      width,
      height,
      output.allowUpscale ? "upscale" : "bounded",
    ].join(":");
    if (previews[output.id]?.signature === signature) return;

    if (previews[output.id]?.url) URL.revokeObjectURL(previews[output.id].url);

    previews = {
      ...previews,
      [output.id]: { signature, url: "", error: "" },
    };
    try {
      const rendered = await previewWithTimeout(
        invoke<PreviewImage>("render_preview", {
          path: output.sourcePath,
          role: "output-preview",
          width,
          height,
          allowUpscale: output.allowUpscale,
        }),
      );
      const bytes = Uint8Array.from(rendered.pngBytes);
      const url = URL.createObjectURL(new Blob([bytes.buffer], { type: "image/png" }));
      const previous = previews[output.id];
      if (previous?.signature !== signature) {
        URL.revokeObjectURL(url);
        return;
      }
      if (previous.url) URL.revokeObjectURL(previous.url);
      previews = {
        ...previews,
        [output.id]: { signature, url, error: "" },
      };
    } catch (error) {
      previews = {
        ...previews,
        [output.id]: { signature, url: "", error: sourceOperationErrorMessage(error) },
      };
    }
  }

  async function previewWithTimeout(request: Promise<PreviewImage>): Promise<PreviewImage> {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    const expired = new Promise<PreviewImage>((_, reject) => {
      timeout = setTimeout(
        () => reject(new Error("Preview rendering timed out. Restart the desktop application.")),
        PREVIEW_TIMEOUT_MS,
      );
    });

    try {
      return await Promise.race([request, expired]);
    } finally {
      if (timeout !== undefined) clearTimeout(timeout);
    }
  }

  function changePlatformFilter(value: string) {
    platformFilter = value;
    scrollTop = 0;
    if (viewportElement) viewportElement.scrollTop = 0;
  }

  function removeOutput(id: string) {
    const preview = previews[id];
    if (preview?.url) URL.revokeObjectURL(preview.url);
    const remaining = { ...previews };
    delete remaining[id];
    previews = remaining;
    onRemove(id);
  }
</script>

<div class="table-toolbar">
  <label>
    <span>Platform filter</span>
    <select
      value={platformFilter}
      onchange={(event) => changePlatformFilter(event.currentTarget.value)}
    >
      <option value="all">All platforms</option>
      {#each platforms as platform (platform)}
        <option value={platform}>{platform}</option>
      {/each}
    </select>
  </label>
  {#if platforms.includes("android")}
    <label>
      <span>Android launcher mask</span>
      <select bind:value={launcherShape}>
        <option value="none">Full bleed</option>
        <option value="circle">Circle</option>
        <option value="squircle">Squircle</option>
        <option value="rounded-square">Rounded square</option>
      </select>
    </label>
  {/if}
  <p>{filteredOutputs.length} output{filteredOutputs.length === 1 ? "" : "s"}</p>
</div>

{#if additionalRoles.length}
  <aside class="role-requirements" aria-labelledby="role-requirements-heading">
    <div>
      <strong id="role-requirements-heading">Additional sources needed</strong>
      <p>Some included outputs cannot use the master artwork alone.</p>
    </div>
    <ul>
      {#each additionalRoles as requirement (requirement.role)}
        <li class:missing={!requirement.configured}>
          <span class="role-name">{requirement.role}</span>
          <span>
            Needed by {requirement.outputIds.length} output{requirement.outputIds.length === 1
              ? ""
              : "s"}
            · {requirement.configured ? "source assigned" : "source still required"}
          </span>
        </li>
      {/each}
    </ul>
  </aside>
{/if}

<div
  class="output-table"
  role="table"
  aria-label="Output review table"
  aria-rowcount={filteredOutputs.length + 1}
  aria-colcount="11"
>
  <div class="table-header table-grid" role="row">
    <span role="columnheader">Include</span>
    <span role="columnheader">Preview</span>
    <span role="columnheader">Platform</span>
    <span role="columnheader">Usage</span>
    <span role="columnheader">Dimensions</span>
    <span role="columnheader">Format</span>
    <span role="columnheader">Filename</span>
    <span role="columnheader">Destination</span>
    <span role="columnheader">Source</span>
    <span role="columnheader">Status</span>
    <span role="columnheader">Actions</span>
  </div>

  <div
    class="table-viewport"
    bind:this={viewportElement}
    style={`height: ${VIEWPORT_HEIGHT}px`}
    onscroll={(event) => (scrollTop = event.currentTarget.scrollTop)}
  >
    <div class="table-spacer" style={`height: ${filteredOutputs.length * ROW_HEIGHT}px`}>
      {#each visibleOutputs as output, offset (output.id)}
        {@const previewLayout = outputPreviewLayout(output.width, output.height)}
        <div
          class="table-row table-grid"
          class:excluded={!output.included}
          role="row"
          aria-rowindex={startIndex + offset + 2}
          style={`transform: translateY(${(startIndex + offset) * ROW_HEIGHT}px)`}
        >
          <div role="cell">
            <label class="compact checkbox-label">
              <input
                type="checkbox"
                checked={output.included}
                aria-label={`Include ${output.filename}`}
                onchange={(event) => onToggle(output.id, event.currentTarget.checked)}
              />
              <span>{output.included ? "Yes" : "No"}</span>
            </label>
          </div>

          <div class="thumbnail checkerboard" role="cell">
            {#if previews[output.id]?.url}
              <div
                class="thumbnail-image"
                style={`width: ${previewLayout.displayWidth}px; height: ${previewLayout.displayHeight}px`}
                title={previewLayout.integerZoom === null
                  ? `${previewLayout.renderWidth}×${previewLayout.renderHeight} preview fitted to the cell`
                  : `${previewLayout.renderWidth}×${previewLayout.renderHeight} preview at ${previewLayout.integerZoom}× integer zoom`}
              >
                <img
                  src={previews[output.id].url}
                  alt={`Preview of ${output.filename} at ${previewLayout.renderWidth} by ${previewLayout.renderHeight} pixels`}
                  width={previewLayout.renderWidth}
                  height={previewLayout.renderHeight}
                  class:pixel-aligned={previewLayout.integerZoom !== null}
                  class:mask-circle={output.platform === "android" && launcherShape === "circle"}
                  class:mask-squircle={output.platform === "android" &&
                    launcherShape === "squircle"}
                  class:mask-rounded-square={output.platform === "android" &&
                    launcherShape === "rounded-square"}
                />
                {#if output.safeZonePercent}
                  <span
                    class="maskable-safe-zone"
                    aria-hidden="true"
                    style={`inset: ${(100 - output.safeZonePercent) / 2}%`}
                  ></span>
                  <span class="sr-only">{output.safeZonePercent}% safe-zone circle shown</span>
                {/if}
              </div>
            {:else}
              <small>{previews[output.id]?.error || "Rendering…"}</small>
            {/if}
          </div>

          <div role="cell"><span class="platform">{output.platform}</span></div>

          <div role="cell">
            {#if output.presetId}
              <p class="preset-value">{output.usage}</p>
            {:else}
              <label class="sr-only" for={`usage-${output.id}`}>Usage</label>
              <textarea
                id={`usage-${output.id}`}
                value={output.usage}
                required
                maxlength="500"
                onchange={(event) => onUpdate(output.id, "usage", event.currentTarget.value)}
              ></textarea>
            {/if}
          </div>

          <div class="dimensions" role="cell">
            {#if output.format === "svg"}
              <span class="preset-value">Scalable</span>
            {:else}
              <label>
                <span>W</span>
                <input
                  type="number"
                  min="1"
                  max="16384"
                  value={output.width}
                  onchange={(event) =>
                    onUpdate(output.id, "width", Number(event.currentTarget.value))}
                />
              </label>
              <span aria-hidden="true">×</span>
              <label>
                <span>H</span>
                <input
                  type="number"
                  min="1"
                  max="16384"
                  value={output.height}
                  onchange={(event) =>
                    onUpdate(output.id, "height", Number(event.currentTarget.value))}
                />
              </label>
            {/if}
          </div>

          <div role="cell"><span class="format">{output.format.toUpperCase()}</span></div>

          <div role="cell">
            {#if output.presetId}
              <span class="preset-value">{output.filename}</span>
            {:else}
              <label class="sr-only" for={`filename-${output.id}`}>Filename</label>
              <input
                id={`filename-${output.id}`}
                value={output.filename}
                required
                maxlength="255"
                onchange={(event) => onUpdate(output.id, "filename", event.currentTarget.value)}
              />
            {/if}
          </div>

          <div role="cell">
            {#if output.presetId}
              <span class="preset-value">{output.destination}</span>
            {:else}
              <label class="sr-only" for={`destination-${output.id}`}>Destination</label>
              <input
                id={`destination-${output.id}`}
                value={output.destination}
                placeholder="package/path"
                maxlength="1024"
                onchange={(event) => onUpdate(output.id, "destination", event.currentTarget.value)}
              />
            {/if}
          </div>

          <div role="cell">
            <strong>{output.sourceLabel}</strong>
            {#if output.sourceOverride}
              <small>Per-output override</small>
            {:else if output.presetId}
              <small>{output.presetId} v{output.presetVersion}</small>
            {:else}
              <small>{output.sourceRole}</small>
            {/if}
            {#if output.sourceRole !== "master" && output.sourceRole !== "override"}
              <small class="additional-role">Additional role: {output.sourceRole}</small>
            {/if}
          </div>

          <div role="cell">
            <span class="status {output.status}">{output.status}</span>
            {#if output.issues.length}
              <ul class="issues">
                {#each output.issues as issue (`${issue.code}:${issue.message}`)}
                  <li>{issue.message}</li>
                {/each}
              </ul>
            {/if}
            {#if output.issues.some((issue) => issue.code === "raster-upscale-forbidden") || output.allowUpscale}
              <label class="compact checkbox-label">
                <input
                  type="checkbox"
                  checked={output.allowUpscale}
                  onchange={(event) => onAllowUpscale(output.id, event.currentTarget.checked)}
                />
                <span>Allow upscale</span>
              </label>
            {/if}
          </div>

          <div class="row-actions" role="cell">
            <button type="button" onclick={() => onReplace(output.id)}>Replace source</button>
            {#if output.sourceOverride}
              <button type="button" onclick={() => onResetSource(output.id)}>Reset source</button>
            {/if}
            {#if !output.presetId}
              <button class="danger" type="button" onclick={() => removeOutput(output.id)}>
                Remove
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .table-toolbar {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.8rem;
  }

  .table-toolbar p {
    margin: 0;
    color: #5d6a62;
    font-size: 0.82rem;
    font-weight: 700;
  }

  .role-requirements {
    display: flex;
    align-items: baseline;
    gap: 0.85rem;
    margin: 0 0 0.8rem;
    border: 1px solid #b9c8bb;
    border-radius: 0.55rem;
    padding: 0.7rem 0.8rem;
    background: #f5f8f2;
    color: #304539;
    font-size: 0.76rem;
  }

  .role-requirements > div {
    flex: 0 0 auto;
  }

  .role-requirements p {
    max-width: 22rem;
    margin: 0.2rem 0 0;
    color: #5d6a62;
    line-height: 1.35;
  }

  .role-requirements ul {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .role-requirements li {
    display: flex;
    gap: 0.35rem;
    border-radius: 999px;
    padding: 0.28rem 0.5rem;
    background: #deece1;
  }

  .role-requirements li.missing {
    background: #f8deda;
    color: #6f2828;
  }

  .role-name {
    font-weight: 850;
  }

  label {
    display: grid;
    gap: 0.3rem;
  }

  label > span {
    color: #33443a;
    font-size: 0.76rem;
    font-weight: 750;
  }

  input,
  select,
  textarea,
  button {
    font: inherit;
  }

  input,
  select,
  textarea {
    width: 100%;
    border: 1px solid #9ba79e;
    border-radius: 0.4rem;
    padding: 0.45rem 0.5rem;
    background: #fff;
    color: #18201d;
  }

  textarea {
    min-height: 4.5rem;
    resize: vertical;
  }

  button {
    border: 1px solid #819081;
    border-radius: 0.4rem;
    padding: 0.42rem 0.55rem;
    background: #fff;
    color: #203128;
    cursor: pointer;
    font-size: 0.72rem;
    font-weight: 700;
  }

  button.danger {
    border-color: #b47878;
    color: #7b2929;
  }

  .output-table {
    overflow-x: auto;
    border: 1px solid rgba(59, 76, 65, 0.25);
    border-radius: 0.7rem;
    background: rgba(249, 250, 246, 0.9);
  }

  .table-grid {
    display: grid;
    grid-template-columns: 5rem 6rem 6rem 12rem 9rem 5rem 11rem 11rem 9rem 14rem 8rem;
    min-width: 102rem;
    gap: 0.7rem;
    align-items: start;
  }

  .table-header {
    position: sticky;
    z-index: 2;
    top: 0;
    border-bottom: 1px solid rgba(59, 76, 65, 0.25);
    padding: 0.72rem;
    background: #e9eee7;
    color: #3a4d40;
    font-size: 0.7rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .table-viewport {
    position: relative;
    overflow-y: auto;
  }

  .table-spacer {
    position: relative;
    min-width: 102rem;
  }

  .table-row {
    position: absolute;
    inset: 0 auto auto 0;
    width: 100%;
    height: 176px;
    border-bottom: 1px solid rgba(59, 76, 65, 0.14);
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.74);
  }

  .table-row.excluded {
    /* Grayscale preserves luminance, so excluded rows read as inactive
       without pushing text below the WCAG AA contrast threshold the way
       opacity dimming did. */
    filter: grayscale(0.85);
    background: #f4f6f2;
  }

  .thumbnail {
    width: 5rem;
    height: 5rem;
    display: grid;
    place-items: center;
    overflow: hidden;
    border: 1px solid #a8b0a9;
    border-radius: 0.35rem;
  }

  .thumbnail-image {
    position: relative;
  }

  .thumbnail-image img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    image-rendering: auto;
  }

  .thumbnail-image img.pixel-aligned {
    image-rendering: pixelated;
  }

  .thumbnail-image img.mask-circle {
    clip-path: circle(50% at 50% 50%);
  }

  .thumbnail-image img.mask-squircle {
    border-radius: 34%;
  }

  .thumbnail-image img.mask-rounded-square {
    border-radius: 22%;
  }

  .maskable-safe-zone {
    position: absolute;
    border: 2px dashed #fff;
    border-radius: 50%;
    box-shadow: 0 0 0 1px rgba(28, 36, 31, 0.82);
    pointer-events: none;
  }

  .checkerboard {
    background-color: #f4f4f0;
    background-image:
      linear-gradient(45deg, #c9cec8 25%, transparent 25%),
      linear-gradient(-45deg, #c9cec8 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #c9cec8 75%),
      linear-gradient(-45deg, transparent 75%, #c9cec8 75%);
    background-position:
      0 0,
      0 6px,
      6px -6px,
      -6px 0;
    background-size: 12px 12px;
  }

  .dimensions {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: end;
    gap: 0.25rem;
  }

  .dimensions input {
    min-width: 0;
  }

  .preset-value {
    margin: 0;
    color: #31463a;
    font-size: 0.74rem;
    line-height: 1.4;
    overflow-wrap: anywhere;
  }

  .platform,
  .format,
  .status {
    display: inline-block;
    border-radius: 999px;
    padding: 0.25rem 0.48rem;
    background: #e3e9e2;
    color: #385140;
    font-size: 0.68rem;
    font-weight: 850;
    text-transform: uppercase;
  }

  .status.ready {
    background: #dff2e5;
    color: #22583a;
  }

  .status.warning {
    background: #fff0c8;
    color: #705019;
  }

  .status.error {
    background: #f8deda;
    color: #7a2929;
  }

  .issues {
    max-height: 4.6rem;
    margin: 0.45rem 0;
    overflow: auto;
    padding-left: 1rem;
    color: #69452a;
    font-size: 0.68rem;
  }

  .row-actions {
    display: grid;
    gap: 0.35rem;
  }

  .compact {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .compact input {
    width: auto;
  }

  .compact span {
    font-size: 0.7rem;
  }

  [role="cell"] > strong,
  [role="cell"] > small {
    display: block;
  }

  [role="cell"] > small.additional-role {
    margin-top: 0.45rem;
    color: #4f3b16;
    font-weight: 800;
  }

  @media (max-width: 760px) {
    .role-requirements {
      align-items: stretch;
      flex-direction: column;
    }
  }

  [role="cell"] > small {
    margin-top: 0.25rem;
    color: #5a675f;
    font-size: 0.68rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
</style>
