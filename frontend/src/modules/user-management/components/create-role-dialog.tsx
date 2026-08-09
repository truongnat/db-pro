import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface CreateRoleDialogProps {
  open: boolean;
  onClose: () => void;
  onCreate: (name: string, login: boolean) => void;
}

export function CreateRoleDialog({ open, onClose, onCreate }: CreateRoleDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [login, setLogin] = useState(true);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    onCreate(name.trim(), login);
    setName("");
    setLogin(true);
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-[384px]">
        <DialogHeader>
          <DialogTitle>{t("userManagement.createRole")}</DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="role-name">{t("userManagement.roleName")}</Label>
            <Input
              id="role-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoFocus
            />
          </div>

          <Label className="flex items-center gap-2 font-normal">
            <Checkbox checked={login} onCheckedChange={(val) => setLogin(val === true)} />
            {t("userManagement.canLogin")}
          </Label>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              {t("common.actions.cancel")}
            </Button>
            <Button type="submit" disabled={!name.trim()}>
              {t("common.actions.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
