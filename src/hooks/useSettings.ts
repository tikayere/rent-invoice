import { useCallback, useEffect, useState } from "react";
import { settingsApi } from "@/services/api";
import type { Settings, SettingsInput } from "@/types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await settingsApi.get();
      setSettings(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const save = useCallback(async (input: SettingsInput) => {
    const updated = await settingsApi.update(input);
    setSettings(updated);
    return updated;
  }, []);

  return { settings, loading, error, reload, save };
}
