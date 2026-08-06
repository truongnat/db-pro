import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

interface TagInputProps {
  tags: string[];
  onChange: (tags: string[]) => void;
}

export function TagInput({ tags, onChange }: TagInputProps) {
  const { t } = useTranslation();
  const [input, setInput] = useState("");

  const addTag = () => {
    const tag = input.trim();
    if (tag && !tags.includes(tag)) {
      onChange([...tags, tag]);
      setInput("");
    }
  };

  const removeTag = (tag: string) => {
    onChange(tags.filter((t) => t !== tag));
  };

  return (
    <div className="flex flex-col gap-2">
      <label className="text-sm font-medium" style={{ color: "var(--color-text)" }}>
        {t("connection.tags")}
      </label>
      <div className="flex flex-wrap items-center gap-1.5">
        {tags.map((tag) => (
          <span
            key={tag}
            className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs"
            style={{
              backgroundColor: "var(--color-surface)",
              color: "var(--color-text)",
              border: "1px solid var(--color-border)",
            }}
          >
            {tag}
            <button
              type="button"
              className="ml-0.5 text-xs hover:opacity-70"
              style={{ color: "var(--color-text-secondary)" }}
              onClick={() => removeTag(tag)}
            >
              ×
            </button>
          </span>
        ))}
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("connection.addTag")}
          className="min-w-[100px] flex-1 rounded-[var(--radius-sm)] border px-2 py-1 text-sm"
          style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              addTag();
            }
            if (e.key === "Backspace" && !input && tags.length > 0) {
              removeTag(tags[tags.length - 1]);
            }
          }}
        />
      </div>
    </div>
  );
}
