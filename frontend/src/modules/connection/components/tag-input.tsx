import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";

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
      <Label htmlFor="connection-tags" className="text-xs font-medium">
        {t("connection.tags")}
      </Label>
      {/*
        Focus ring uses border-color (not outline) because outline is clipped
        by the dialog's overflow-y-auto scroll container. Border is always
        within the element's box and can never be clipped by overflow.
      */}
      <div className="flex min-h-8 flex-wrap items-center gap-1.5 rounded-lg border border-input bg-transparent px-2.5 py-1 transition-colors focus-within:border-primary">
        {tags.map((tag) => (
          <Badge key={tag} variant="secondary" className="gap-1">
            {tag}
            <Button
              type="button"
              variant="ghost"
              size="icon-xs"
              className="size-3 p-0 text-[var(--app-text-muted)] hover:opacity-70"
              onClick={() => removeTag(tag)}
              aria-label={`Remove tag ${tag}`}
            >
              ×
            </Button>
          </Badge>
        ))}
        <input
          id="connection-tags"
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("connection.addTag")}
          className="min-w-[100px] flex-1 border-0 bg-transparent p-0 text-sm outline-none placeholder:text-muted-foreground focus-visible:outline-none"
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
