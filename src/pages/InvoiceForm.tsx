import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Save, Printer, FileDown } from "lucide-react";
import { Card, CardBody, CardHeader } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";
import { Select } from "@/components/ui/Select";
import { Button } from "@/components/ui/Button";
import { PageLoader } from "@/components/ui/Loader";
import { useTenants } from "@/hooks/useTenants";
import { useSettings } from "@/hooks/useSettings";
import { useToast } from "@/context/ToastContext";
import { invoiceSchema, type InvoiceFormValues } from "@/utils/validation";
import { formatCurrency, monthLabel, todayIso, addDaysIso } from "@/utils/format";
import { invoicesApi, pdfApi } from "@/services/api";
import { pickPdfSaveTarget, notify } from "@/services/tauri";
import type { Invoice } from "@/types";

const MONTHS = Array.from({ length: 12 }, (_, i) => i + 1);
const CURRENT_YEAR = new Date().getFullYear();
const YEARS = Array.from({ length: 6 }, (_, i) => CURRENT_YEAR - 2 + i);

export function InvoiceFormPage() {
  const { id } = useParams();
  const isEdit = Boolean(id);
  const navigate = useNavigate();
  const toast = useToast();
  const { tenants, loading: tenantsLoading } = useTenants();
  const { settings } = useSettings();
  const [nextNumber, setNextNumber] = useState<string>("");
  const [loadingInvoice, setLoadingInvoice] = useState(isEdit);
  const [invoiceId, setInvoiceId] = useState<number | null>(id ? Number(id) : null);

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<InvoiceFormValues>({
    resolver: zodResolver(invoiceSchema),
    defaultValues: {
      tenant_id: 0,
      property_address: "",
      description: "",
      billing_month: new Date().getMonth() + 1,
      billing_year: CURRENT_YEAR,
      issue_date: todayIso(),
      due_date: addDaysIso(todayIso(), 7),
      rent_amount: 0,
      water_charge: 0,
      electricity_charge: 0,
      other_charges: 0,
      discount: 0,
      amount_paid: 0,
      payment_method: "cash",
      observations: "",
    },
  });

  useEffect(() => {
    if (!isEdit) {
      invoicesApi.nextNumber().then(setNextNumber).catch(() => setNextNumber(""));
    }
  }, [isEdit]);

  useEffect(() => {
    if (isEdit && id) {
      setLoadingInvoice(true);
      invoicesApi
        .get(Number(id))
        .then((inv: Invoice) => {
          setInvoiceId(inv.id);
          setNextNumber(inv.invoice_number);
          reset({
            tenant_id: inv.tenant_id,
            property_address: inv.property_address,
            description: inv.description ?? "",
            billing_month: inv.billing_month,
            billing_year: inv.billing_year,
            issue_date: inv.issue_date,
            due_date: inv.due_date,
            rent_amount: inv.rent_amount,
            water_charge: inv.water_charge,
            electricity_charge: inv.electricity_charge,
            other_charges: inv.other_charges,
            discount: inv.discount,
            amount_paid: inv.amount_paid,
            payment_method: inv.payment_method,
            observations: inv.observations ?? "",
          });
        })
        .catch((e) => toast.error(String(e)))
        .finally(() => setLoadingInvoice(false));
    }
  }, [isEdit, id, reset, toast]);

  const values = watch();
  const currency = settings?.currency ?? "XOF";

  const total = useMemo(() => {
    const rent = Number(values.rent_amount) || 0;
    const water = Number(values.water_charge) || 0;
    const elec = Number(values.electricity_charge) || 0;
    const other = Number(values.other_charges) || 0;
    const discount = Number(values.discount) || 0;
    return Math.max(0, rent + water + elec + other - discount);
  }, [values.rent_amount, values.water_charge, values.electricity_charge, values.other_charges, values.discount]);

  const balanceDue = useMemo(() => {
    const paid = Number(values.amount_paid) || 0;
    return Math.max(0, total - paid);
  }, [total, values.amount_paid]);

  const status = useMemo(() => {
    const paid = Number(values.amount_paid) || 0;
    if (paid <= 0) return "unpaid";
    if (paid >= total && total > 0) return "paid";
    return "partially_paid";
  }, [total, values.amount_paid]);

  async function onSubmit(values: InvoiceFormValues) {
    const payload = {
      ...values,
      description: values.description || null,
      observations: values.observations || null,
    };
    try {
      let saved: Invoice;
      if (isEdit && invoiceId) {
        saved = await invoicesApi.update(invoiceId, payload);
      } else {
        saved = await invoicesApi.create(payload);
        setInvoiceId(saved.id);
      }
      toast.success(`Facture ${saved.invoice_number} enregistree`);
      navigate(`/invoices/${saved.id}/edit`, { replace: true });
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleGeneratePdf() {
    if (!invoiceId) {
      toast.info("Enregistrez d'abord la facture avant de generer le PDF.");
      return;
    }
    const dest = await pickPdfSaveTarget(`${nextNumber || "facture"}.pdf`);
    if (!dest) return;
    try {
      await pdfApi.generateToPath(invoiceId, dest);
      await notify("Le PDF de la facture a ete genere avec succes.", "Facture generee");
    } catch (e) {
      toast.error(`Echec de la generation du PDF : ${String(e)}`);
    }
  }

  if (loadingInvoice || tenantsLoading) return <PageLoader label="Chargement de la facture..." />;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-6 max-w-5xl">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-slate-500">Numero de facture</p>
          <p className="text-lg font-semibold text-brand-700 dark:text-brand-400">{nextNumber || "..."}</p>
        </div>
        <div className="flex gap-2">
          <Button type="button" variant="outline" onClick={handleGeneratePdf} disabled={!invoiceId}>
            <Printer size={16} /> Apercu / Imprimer
          </Button>
          <Button type="button" variant="outline" onClick={handleGeneratePdf} disabled={!invoiceId}>
            <FileDown size={16} /> Telecharger PDF
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader title="Locataire et bien" />
        <CardBody className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Select label="Locataire" {...register("tenant_id")} error={errors.tenant_id?.message}>
            <option value={0}>Selectionner un locataire...</option>
            {tenants.map((t) => (
              <option key={t.id} value={t.id}>
                {t.first_name} {t.last_name}
              </option>
            ))}
          </Select>
          <Input label="Adresse du bien" {...register("property_address")} error={errors.property_address?.message} />
          <div className="sm:col-span-2">
            <Textarea label="Description (optionnel)" rows={2} {...register("description")} />
          </div>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Periode et dates" />
        <CardBody className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <Select label="Mois concerne" {...register("billing_month")}>
            {MONTHS.map((m) => (
              <option key={m} value={m}>
                {monthLabel(m)}
              </option>
            ))}
          </Select>
          <Select label="Annee" {...register("billing_year")}>
            {YEARS.map((y) => (
              <option key={y} value={y}>
                {y}
              </option>
            ))}
          </Select>
          <Input label="Date d'emission" type="date" {...register("issue_date")} error={errors.issue_date?.message} />
          <Input label="Date d'echeance" type="date" {...register("due_date")} error={errors.due_date?.message} />
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Montants" subtitle="Le total et le solde sont calcules automatiquement." />
        <CardBody className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <Input label={`Loyer (${currency})`} type="number" step="0.01" {...register("rent_amount")} error={errors.rent_amount?.message} />
          <Input label={`Eau (${currency})`} type="number" step="0.01" {...register("water_charge")} />
          <Input label={`Electricite (${currency})`} type="number" step="0.01" {...register("electricity_charge")} />
          <Input label={`Autres frais (${currency})`} type="number" step="0.01" {...register("other_charges")} />
          <Input label={`Remise (${currency})`} type="number" step="0.01" {...register("discount")} error={errors.discount?.message} />
          <Input label={`Montant paye (${currency})`} type="number" step="0.01" {...register("amount_paid")} />
        </CardBody>
        <CardBody className="border-t border-slate-200 dark:border-slate-800 grid grid-cols-1 gap-4 sm:grid-cols-3 bg-slate-50 dark:bg-slate-900/40 rounded-b-xl">
          <div>
            <p className="text-xs text-slate-500">Total</p>
            <p className="text-xl font-semibold text-slate-900 dark:text-slate-100">{formatCurrency(total, currency)}</p>
          </div>
          <div>
            <p className="text-xs text-slate-500">Reste a payer</p>
            <p className="text-xl font-semibold text-slate-900 dark:text-slate-100">{formatCurrency(balanceDue, currency)}</p>
          </div>
          <div>
            <p className="text-xs text-slate-500">Statut</p>
            <p className="text-xl font-semibold capitalize text-slate-900 dark:text-slate-100">
              {status === "paid" ? "Paye" : status === "partially_paid" ? "Partiellement paye" : "Non paye"}
            </p>
          </div>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Paiement" />
        <CardBody className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Select label="Mode de paiement" {...register("payment_method")}>
            <option value="cash">Especes</option>
            <option value="bank_transfer">Virement</option>
            <option value="mobile_money">Mobile Money</option>
            <option value="check">Cheque</option>
            <option value="other">Autre</option>
          </Select>
          <div className="sm:col-span-2">
            <Textarea label="Observations (optionnel)" rows={2} {...register("observations")} />
          </div>
        </CardBody>
      </Card>

      <div className="sticky bottom-0 flex justify-end gap-3 border-t border-slate-200 dark:border-slate-800 bg-slate-50/90 dark:bg-slate-950/90 backdrop-blur py-4">
        <Button type="button" variant="outline" onClick={() => navigate(-1)}>
          Annuler
        </Button>
        <Button type="submit" loading={isSubmitting}>
          <Save size={16} /> {isEdit ? "Enregistrer les modifications" : "Creer la facture"}
        </Button>
      </div>
    </form>
  );
}
