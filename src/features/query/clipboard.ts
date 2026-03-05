export async function copyTextToClipboard(value: string): Promise<void> {
  if (
    typeof navigator !== "undefined" &&
    navigator.clipboard &&
    typeof navigator.clipboard.writeText === "function"
  ) {
    await navigator.clipboard.writeText(value);
    return;
  }

  if (typeof document === "undefined") {
    throw new Error("Clipboard API is not available in this environment.");
  }

  const textarea = document.createElement("textarea");
  textarea.value = value;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();

  try {
    const succeeded = document.execCommand("copy");
    if (!succeeded) {
      throw new Error("Unable to copy to clipboard.");
    }
  } finally {
    document.body.removeChild(textarea);
  }
}

function normalizeTabValue(value: string): string {
  return value.replace(/\t/g, " ").replace(/\r?\n/g, " ");
}

export function toTabDelimited(
  columns: string[],
  rows: string[][],
  includeHeader: boolean,
): string {
  const lines: string[] = [];

  if (includeHeader) {
    lines.push(columns.map((item) => normalizeTabValue(item)).join("\t"));
  }

  for (const row of rows) {
    lines.push(row.map((item) => normalizeTabValue(item)).join("\t"));
  }

  return lines.join("\n");
}
