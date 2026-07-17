import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Image as ImageIcon, Save, DownloadCloud, UploadCloud } from "lucide-react";
import { Card, CardBody, CardHeader } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";
import { Select } from "@/components/ui/Select";
import { Button } from "@/components/ui/Button";
import { PageLoader } from "@/components/ui/Loader";
import { useSettings } from "@/hooks/useSettings";
import { useToast } from "@/context/ToastContext";
import { useTheme } from "@/context/ThemeContext";
import { settingsSchema, type SettingsFormValues } from "@/utils/validation";
import { pickImageFile, pickBackupSaveTarget, pickBackupToRestore, confirmDestructive, toAssetSrc } from "@/services/tauri";
import { backupApi } from "@/services/api";

const CURRENCIES = ["XOF", "XAF", "EUR", "USD", "GBP", "MAD", "GNF", "NGN", "GHS"];

export function SettingsPage() {
  const { settings, loading, save } = useSettings();
  const { setTheme } = useTheme();
  const toast = useToast();

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<SettingsFormValues>({
    resolver: zodResolver(settingsSchema),
    defaultValues: {
      full_name: "",
      address: "",
      phone: "",
      email: "",
      city: "",
      country: "",
      currency: "XOF",
      invoice_prefix: "LOY",
      date_format: "DD/MM/YYYY",
      language: "fr",
      theme: "light",
      invoice_template: "classic",
    },
  });

  useEffect(() => {
    if (settings) {
      reset({
        full_name: settings.full_name,
        company_name: settings.company_name,
        address: settings.address,
        phone: settings.phone,
        email: settings.email,
        city: settings.city,
        country: settings.country,
        currency: settings.currency,
        logo_path: settings.logo_path,
        signature_path: settings.signature_path,
        tax_number: settings.tax_number,
        iban: settings.iban,
        additional_info: settings.additional_info,
        invoice_prefix: settings.invoice_prefix,
        date_format: settings.date_format,
        language: settings.language,
        theme: settings.theme,
        invoice_template: settings.invoice_template,
      });
    }
  }, [settings, reset]);

  const logoPath = watch("logo_path");
  const signaturePath = watch("signature_path");

  async function onSubmit(values: SettingsFormValues) {
    try {
      await save(values);
      setTheme(values.theme);
      toast.success("Parametres enregistres avec succes");
    } catch (e) {
      toast.error(`Echec de l'enregistrement : ${String(e)}`);
    }
  }

  async function handlePickImage(field: "logo_path" | "signature_path") {
    const path = await pickImageFile();
    if (path) setValue(field, path, { shouldDirty: true });
  }

  async function handleExportDb() {
    const dest = await pickBackupSaveTarget(`sauvegarde-loyers-${Date.now()}.db`);
    if (!dest) return;
    try {
      await backupApi.exportTo(dest);
      toast.success("Sauvegarde exportee avec succes");
    } catch (e) {
      toast.error(`Echec de l'export : ${String(e)}`);
    }
  }

  async function handleImportDb() {
    const src = await pickBackupToRestore();
    if (!src) return;
    const confirmed = await confirmDestructive(
      "Restaurer cette sauvegarde remplacera toutes les donnees actuelles. Une sauvegarde de securite sera creee automatiquement avant l'import. Continuer ?"
    );
    if (!confirmed) return;
    try {
      await backupApi.importFrom(src);
      toast.success("Sauvegarde importee. Redemarrez l'application pour appliquer les changements.");
    } catch (e) {
      toast.error(`Echec de l'import : ${String(e)}`);
    }
  }

  if (loading) return <PageLoader label="Chargement des parametres..." />;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-6 max-w-4xl">
      <Card>
        <CardHeader title="Informations du bailleur" subtitle="Ces informations apparaissent sur chaque facture generee." />
        <CardBody className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Input label="Nom complet" {...register("full_name")} error={errors.full_name?.message} />
          <Input label="Nom de la societe (optionnel)" {...register("company_name")} />
          <Input label="Telephone" {...register("phone")} error={errors.phone?.message} />
          <Input label="Email" type="email" {...register("email")} error={errors.email?.message} />
          <Input label="Ville" {...register("city")} error={errors.city?.message} />
          <Input label="Pays" {...register("country")} error={errors.country?.message} />
          <div className="sm:col-span-2">
            <Textarea label="Adresse" rows={2} {...register("address")} error={errors.address?.message} />
          </div>
          <Input label="Numero fiscal (optionnel)" {...register("tax_number")} />
          <Input label="IBAN (optionnel)" {...register("iban")} />
          <div className="sm:col-span-2">
            <Textarea label="Informations complementaires" rows={2} {...register("additional_info")} />
          </div>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Logo et signature" subtitle="Images utilisees dans l'en-tete et le pied de page du PDF." />
        <CardBody className="grid grid-cols-1 gap-6 sm:grid-cols-2">
          <div className="flex flex-col gap-2">
            <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Logo</span>
            <div className="flex h-28 w-full items-center justify-center overflow-hidden rounded-lg border border-dashed border-slate-300 dark:border-slate-700 bg-slate-50 dark:bg-slate-950">
              {logoPath ? (
                <img src={toAssetSrc(logoPath)} alt="Logo" className="max-h-full max-w-full object-contain p-2" />
              ) : (
                <ImageIcon className="text-slate-300" size={28} />
              )}
            </div>
            <Button type="button" variant="outline" size="sm" onClick={() => handlePickImage("logo_path")}>
              Choisir un logo
            </Button>
          </div>
          <div className="flex flex-col gap-2">
            <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Signature</span>
            <div className="flex h-28 w-full items-center justify-center overflow-hidden rounded-lg border border-dashed border-slate-300 dark:border-slate-700 bg-slate-50 dark:bg-slate-950">
              {signaturePath ? (
                <img src={toAssetSrc(signaturePath)} alt="Signature" className="max-h-full max-w-full object-contain p-2" />
              ) : (
                <ImageIcon className="text-slate-300" size={28} />
              )}
            </div>
            <Button type="button" variant="outline" size="sm" onClick={() => handlePickImage("signature_path")}>
              Choisir une signature
            </Button>
          </div>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Preferences" subtitle="Devise, format de date, numerotation et apparence." />
        <CardBody className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Select label="Devise" {...register("currency")}>
            {CURRENCIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </Select>
          <Select label="Format de date" {...register("date_format")}>
            <option value="DD/MM/YYYY">JJ/MM/AAAA</option>
            <option value="MM/DD/YYYY">MM/JJ/AAAA</option>
            <option value="YYYY-MM-DD">AAAA-MM-JJ</option>
          </Select>
          <Input
            label="Prefixe des factures"
            hint="Exemple : LOY donnera LOY-2026-000001"
            {...register("invoice_prefix")}
            error={errors.invoice_prefix?.message}
          />
          <Select label="Theme" {...register("theme")}>
            <option value="light">Clair</option>
            <option value="dark">Sombre</option>
          </Select>
          <Select label="Langue" {...register("language")} disabled>
            <option value="fr">Francais</option>
          </Select>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Modele de facture" subtitle="Choisissez l'apparence du PDF genere pour vos factures." />
        <CardBody className="grid grid-cols-1 gap-2 sm:max-w-xs">
          <Select label="Modele" {...register("invoice_template")}>
            <option value="classic">Classique - bleu, en-tete sombre</option>
            <option value="modern">Moderne - vert sarcelle, accents marques</option>
            <option value="minimal">Minimaliste - sobre, sans couleur</option>
          </Select>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="Sauvegarde et restauration" subtitle="Toutes les donnees restent stockees localement sur cet ordinateur." />
        <CardBody className="flex flex-wrap gap-3">
          <Button type="button" variant="outline" onClick={handleExportDb}>
            <DownloadCloud size={16} /> Exporter la base de donnees
          </Button>
          <Button type="button" variant="outline" onClick={handleImportDb}>
            <UploadCloud size={16} /> Importer une sauvegarde
          </Button>
        </CardBody>
      </Card>

      <div className="sticky bottom-0 flex justify-end gap-3 border-t border-slate-200 dark:border-slate-800 bg-slate-50/90 dark:bg-slate-950/90 backdrop-blur py-4">
        <Button type="submit" loading={isSubmitting}>
          <Save size={16} /> Enregistrer les parametres
        </Button>
      </div>
    </form>
  );
}
