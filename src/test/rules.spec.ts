import { describe, expect, it } from "vitest";
import {
  createEmptyRule,
  formatBytes,
  normalizeExtensions,
  splitList,
  validateRule,
} from "@/utils/rules";

describe("rules utilities", () => {
  it("splits comma-separated lists and removes duplicates and whitespace", () => {
    expect(splitList("pdf, docx, pdf, txt,   ")).toEqual(["pdf", "docx", "txt"]);
  });

  it("normalizes extensions by removing dots and converting to lowercase", () => {
    expect(normalizeExtensions(".PDF, .PNG, JPG, .Zip")).toEqual([
      "pdf",
      "png",
      "jpg",
      "zip",
    ]);
  });

  it("formats bytes into human readable units", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(-10)).toBe("0 B");
    expect(formatBytes(500)).toBe("500 B");
    expect(formatBytes(1024)).toBe("1.0 KiB");
    expect(formatBytes(1024 * 1024 * 5)).toBe("5.0 MiB");
    expect(formatBytes(1024 * 1024 * 1024 * 2.5)).toBe("2.5 GiB");
  });

  it("validates rule inputs correctly", () => {
    const emptyRule = createEmptyRule();
    const issues = validateRule(emptyRule);

    expect(issues.some((issue) => issue.field === "name")).toBe(true);
    expect(issues.some((issue) => issue.field === "destination")).toBe(true);
    expect(issues.some((issue) => issue.field === "extensions")).toBe(true);

    const validRule = {
      ...emptyRule,
      name: "Documents",
      sources: ["C:\\Downloads"],
      destination: "C:\\Docs",
      extensions: ["pdf", "docx"],
    };

    expect(validateRule(validRule)).toEqual([]);
  });

  it("detects invalid size constraints when min > max", () => {
    const rule = {
      ...createEmptyRule(),
      name: "Big Files",
      sources: ["C:\\Downloads"],
      destination: "C:\\Archive",
      extensions: ["zip"],
      minSizeBytes: 5000,
      maxSizeBytes: 1000,
    };

    const issues = validateRule(rule);
    expect(issues).toEqual([{ field: "size", code: "invalidSize" }]);
  });
});
