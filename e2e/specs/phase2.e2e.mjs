// SPDX-License-Identifier: MIT OR Apache-2.0
import assert from "node:assert/strict";

import { AxeBuilder } from "@axe-core/webdriverio";

describe("v1 desktop accessibility and workflow smoke", () => {
  it("connects to the Rust core and exposes the expected landmarks and output table", async () => {
    const heading = await $("h1=Imaguncula");
    await heading.waitForDisplayed();
    await browser.waitUntil(async () => (await $(".core-status").getText()).includes("connected"), {
      timeoutMsg: "desktop core did not connect",
    });

    const table = await $('[role="table"][aria-label="Output review table"]');
    await table.waitForDisplayed();

    const headers = await table.$$('[role="columnheader"]');
    assert.equal(headers.length, 11);
    const viewport = await browser.execute(() => {
      const element = document.querySelector(".table-viewport");
      return {
        clientHeight: element?.clientHeight ?? 0,
        visibleRows: element?.querySelectorAll('[role="row"]').length ?? 0,
      };
    });
    assert.equal(viewport.clientHeight, 560);
    assert.ok(viewport.visibleRows > 0);
    assert.equal((await $$("main")).length, 1);
    assert.equal((await $$("header")).length, 1);
    assert.equal((await $$('nav[aria-label="Project actions"]')).length, 1);
    assert.equal((await $$("footer")).length, 1);
    assert.ok((await $$("section[aria-labelledby]")).length >= 3);
    assert.equal(await $("button=Create").isEnabled(), true);
  });

  it("links additional source requirements to their output types", async () => {
    const presetCard = await $('//label[contains(., "Android launcher and Play Store")]');
    const androidPreset = await presetCard.$('input[type="checkbox"]');
    await androidPreset.scrollIntoView();
    await androidPreset.click();

    const presetNotice = await presetCard.$(".preset-source-requirement");
    await presetNotice.waitForDisplayed();
    assert.match(
      await presetNotice.getText(),
      /Requires additional sources: foreground and background/,
    );

    const requirementNotice = await $("#role-requirements-heading");
    await requirementNotice.waitForDisplayed();
    assert.equal(await requirementNotice.getText(), "Additional sources needed");

    const foregroundCard = await $(
      '//article[contains(@class, "role-source-card")][.//h3[normalize-space()="foreground"]]',
    );
    await foregroundCard.waitForDisplayed();
    assert.match(
      (await foregroundCard.getText()).replace(/\s+/g, " "),
      /Required for Android launcher and Play Store/,
    );
  });

  it("supports deterministic keyboard entry and activation", async () => {
    await browser.execute(() => {
      document.body.tabIndex = -1;
      document.body.focus();
    });

    for (const expected of ["New", "Open", "Save"]) {
      await browser.keys(["Tab"]);
      const activeName = await browser.execute(() =>
        (document.activeElement?.textContent ?? "").replace(/\s+/g, " ").trim(),
      );
      assert.equal(activeName, expected);
    }

    const addOutput = await $("button=Add custom output");
    await addOutput.scrollIntoView();
    await browser.execute((element) => element.focus(), addOutput);
    await browser.keys(["Enter"]);
    const customUsage = await $("#usage-custom-1");
    await customUsage.waitForDisplayed();
    assert.equal(await customUsage.isEnabled(), true);
  });

  it("gives every visible interactive control an accessible name", async () => {
    const unnamed = await browser.execute(() => {
      const controls = [...document.querySelectorAll("button, input, select, textarea")];
      return controls
        .filter((element) => {
          const style = window.getComputedStyle(element);
          return !element.disabled && style.display !== "none" && style.visibility !== "hidden";
        })
        .filter((element) => {
          const labelledBy = element.getAttribute("aria-labelledby");
          const labelledText = labelledBy
            ? labelledBy
                .split(/\s+/)
                .map((id) => document.getElementById(id)?.textContent ?? "")
                .join(" ")
            : "";
          const labels =
            "labels" in element
              ? [...element.labels].map((label) => label.textContent ?? "").join(" ")
              : "";
          const name = [
            element.getAttribute("aria-label") ?? "",
            labelledText,
            labels,
            element.tagName === "BUTTON" ? (element.textContent ?? "") : "",
            element.getAttribute("title") ?? "",
          ]
            .join(" ")
            .replace(/\s+/g, " ")
            .trim();
          return name.length === 0;
        })
        .map((element) => element.outerHTML.slice(0, 240));
    });
    assert.deepEqual(unnamed, []);
  });

  it("has no automatic WCAG A or AA violations in the visible v1 surface", async () => {
    const results = await new AxeBuilder({ client: browser })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
      .analyze();
    const failures = results.violations.map((violation) => ({
      id: violation.id,
      impact: violation.impact,
      help: violation.help,
      targets: violation.nodes.map((node) => node.target),
    }));
    assert.deepEqual(failures, []);
  });
});
