import { useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Search, Plus, Pencil, Trash2, Users } from "lucide-react";
import { Card } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";
import { PageLoader } from "@/components/ui/Loader";
import { EmptyState } from "@/components/ui/EmptyState";
import { useTenants } from "@/hooks/useTenants";
import { useToast } from "@/context/ToastContext";
import { confirmDestructive } from "@/services/tauri";
import { tenantSchema, type TenantFormValues } from "@/utils/validation";
import type { Tenant } from "@/types";

export function TenantsPage() {
  const [search, setSearch] = useState("");
  const { tenants, loading, create, update, remove } = useTenants(search || undefined);
  const [modalOpen, setModalOpen] = useState(false);
  const [editing, setEditing] = useState<Tenant | null>(null);
  const toast = useToast();

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<TenantFormValues>({ resolver: zodResolver(tenantSchema) });

  function openCreate() {
    setEditing(null);
    reset({ first_name: "", last_name: "", phone: "", email: "", address: "", id_number: "", profession: "", notes: "" });
    setModalOpen(true);
  }

  function openEdit(t: Tenant) {
    setEditing(t);
    reset({
      first_name: t.first_name,
      last_name: t.last_name,
      phone: t.phone,
      email: t.email ?? "",
      address: t.address,
      id_number: t.id_number ?? "",
      profession: t.profession ?? "",
      notes: t.notes ?? "",
    });
    setModalOpen(true);
  }

  async function onSubmit(values: TenantFormValues) {
    const payload = { ...values, email: values.email || null };
    try {
      if (editing) {
        await update(editing.id, payload);
        toast.success("Locataire mis a jour");
      } else {
        await create(payload);
        toast.success("Locataire ajoute");
      }
      setModalOpen(false);
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleDelete(t: Tenant) {
    const confirmed = await confirmDestructive(
      `Supprimer ${t.first_name} ${t.last_name} ? Cette action est irreversible.`
    );
    if (!confirmed) return;
    try {
      await remove(t.id);
      toast.success("Locataire supprime");
    } catch (e) {
      toast.error(String(e));
    }
  }

  const sorted = useMemo(
    () => [...tenants].sort((a, b) => a.last_name.localeCompare(b.last_name)),
    [tenants]
  );

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="relative">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Rechercher un locataire..."
            className="w-72 rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 py-2 pl-9 pr-3 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
        </div>
        <Button onClick={openCreate}>
          <Plus size={16} /> Ajouter un locataire
        </Button>
      </div>

      {loading ? (
        <PageLoader />
      ) : sorted.length === 0 ? (
        <EmptyState
          icon={<Users size={40} />}
          title="Aucun locataire"
          description="Ajoutez votre premier locataire pour commencer a generer des factures."
          action={
            <Button onClick={openCreate} size="sm" className="mt-2">
              <Plus size={16} /> Ajouter un locataire
            </Button>
          }
        />
      ) : (
        <Card className="overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-slate-50 dark:bg-slate-800/50 text-left text-xs uppercase tracking-wide text-slate-500">
              <tr>
                <th className="px-4 py-3">Nom</th>
                <th className="px-4 py-3">Telephone</th>
                <th className="px-4 py-3">Email</th>
                <th className="px-4 py-3">Adresse</th>
                <th className="px-4 py-3">Factures</th>
                <th className="px-4 py-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
              {sorted.map((t) => (
                <tr key={t.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/30">
                  <td className="px-4 py-3 font-medium text-slate-900 dark:text-slate-100">
                    {t.first_name} {t.last_name}
                  </td>
                  <td className="px-4 py-3 text-slate-600 dark:text-slate-400">{t.phone}</td>
                  <td className="px-4 py-3 text-slate-600 dark:text-slate-400">{t.email ?? "-"}</td>
                  <td className="px-4 py-3 max-w-xs truncate text-slate-600 dark:text-slate-400">{t.address}</td>
                  <td className="px-4 py-3 text-slate-600 dark:text-slate-400">{t.invoice_count ?? 0}</td>
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-1">
                      <button
                        onClick={() => openEdit(t)}
                        className="rounded-md p-1.5 text-slate-400 hover:bg-slate-100 hover:text-brand-600 dark:hover:bg-slate-800"
                      >
                        <Pencil size={16} />
                      </button>
                      <button
                        onClick={() => handleDelete(t)}
                        className="rounded-md p-1.5 text-slate-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950"
                      >
                        <Trash2 size={16} />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? "Modifier le locataire" : "Nouveau locataire"} size="lg">
        <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Input label="Prenom" {...register("first_name")} error={errors.first_name?.message} />
          <Input label="Nom" {...register("last_name")} error={errors.last_name?.message} />
          <Input label="Telephone" {...register("phone")} error={errors.phone?.message} />
          <Input label="Email (optionnel)" type="email" {...register("email")} error={errors.email?.message as string | undefined} />
          <Input label="Numero de piece (optionnel)" {...register("id_number")} />
          <Input label="Profession (optionnel)" {...register("profession")} />
          <div className="sm:col-span-2">
            <Textarea label="Adresse" rows={2} {...register("address")} error={errors.address?.message} />
          </div>
          <div className="sm:col-span-2">
            <Textarea label="Notes (optionnel)" rows={2} {...register("notes")} />
          </div>
          <div className="sm:col-span-2 flex justify-end gap-3 pt-2">
            <Button type="button" variant="outline" onClick={() => setModalOpen(false)}>
              Annuler
            </Button>
            <Button type="submit" loading={isSubmitting}>
              {editing ? "Enregistrer" : "Ajouter"}
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
