import { Link } from "react-router-dom";

import { useSettingsContext } from "@/lib/settings-provider";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Label } from "@mizan/ui/components/ui/label";
import { Switch } from "@mizan/ui/components/ui/switch";
import { toast } from "@mizan/ui/components/ui/use-toast";
import { useState } from "react";

export function IslamicModeSettings() {
  const { settings, updateSettings: updateSettingsContext } = useSettingsContext();
  const [isSaving, setIsSaving] = useState(false);

  const handleToggle = (enabled: boolean) => {
    if (!settings) return;
    setIsSaving(true);
    updateSettingsContext({ shariahModeEnabled: enabled })
      .catch(() => {
        toast({
          title: "Error",
          description: "Failed to update Islamic mode settings. Please try again.",
          variant: "destructive",
        });
      })
      .finally(() => {
        setIsSaving(false);
      });
  };

  if (!settings) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Optional Islamic Finance</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5">
            <Label htmlFor="shariah-mode-enabled" className="text-base">
              Enable Islamic finance tools
            </Label>
            <p className="text-muted-foreground text-xs">
              Adds screening, zakat, and purification tools as an optional overlay. Mizan does not
              provide official certification or fatwa rulings.
            </p>
          </div>
          <Switch
            id="shariah-mode-enabled"
            checked={settings.shariahModeEnabled}
            onCheckedChange={handleToggle}
            disabled={isSaving}
          />
        </div>
        {settings.shariahModeEnabled && (
          <Link to="/shariah-screening" className="text-primary text-sm font-medium underline">
            Open screening page
          </Link>
        )}
      </CardContent>
    </Card>
  );
}
