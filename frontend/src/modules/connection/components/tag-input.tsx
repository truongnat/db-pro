import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

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
    <div className="flex flex-col gap-1.5">
      <label className="text-xs font-medium text-foreground">
        {t("connection.tags")}
      </label>
      <div className="flex flex-wrap items-center gap-1.5 rounded-lg border border-input bg-transparent px-2 py-1.5 focus-within:outline focus-within:outline-[var(--focus-ring-width)] focus-within:outline-offset-[var(--focus-ring-offset)] focus-within:outline-[var(--focus-ring-color)]">
        {tags.map((tag) => (
          <Badge key={tag} variant="secondary" className="gap-1">
            {tag}
            <Button
              type="button"
              variant="ghost"
              size="icon-xs"
              className="size-3 p-0 text-muted-foreground hover:opacity-70"
              onClick={() => removeTag(tag)}
              aria-label={`Remove tag ${tag}`}
            >
              ×
            </Button>
          </Badge>
        ))}
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("connection.addTag")}
          className="min-w-[100px] flex-1 border-0 bg-transparent px-0 py-0 text-sm text-foreground outline-none placeholder:text-muted-foreground"
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
