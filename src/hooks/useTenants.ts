import { useCallback, useEffect, useState } from "react";
import { tenantsApi } from "@/services/api";
import type { Tenant, TenantInput } from "@/types";

export function useTenants(search?: string) {
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await tenantsApi.list(search);
      setTenants(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [search]);

  useEffect(() => {
    reload();
  }, [reload]);

  const create = useCallback(async (input: TenantInput) => {
    const created = await tenantsApi.create(input);
    setTenants((prev) => [created, ...prev]);
    return created;
  }, []);

  const update = useCallback(async (id: number, input: TenantInput) => {
    const updated = await tenantsApi.update(id, input);
    setTenants((prev) => prev.map((t) => (t.id === id ? updated : t)));
    return updated;
  }, []);

  const remove = useCallback(async (id: number) => {
    await tenantsApi.remove(id);
    setTenants((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return { tenants, loading, error, reload, create, update, remove };
}
