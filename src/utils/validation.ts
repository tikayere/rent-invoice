import { z } from "zod";

export const settingsSchema = z.object({
  full_name: z.string().min(2, "Le nom complet est requis (2 caracteres min.)"),
  company_name: z.string().optional().nullable(),
  address: z.string().min(3, "L'adresse est requise"),
  phone: z.string().min(6, "Le telephone est requis"),
  email: z.string().email("Adresse email invalide"),
  city: z.string().min(1, "La ville est requise"),
  country: z.string().min(1, "Le pays est requis"),
  currency: z.string().min(1, "La devise est requise"),
  logo_path: z.string().optional().nullable(),
  signature_path: z.string().optional().nullable(),
  tax_number: z.string().optional().nullable(),
  iban: z.string().optional().nullable(),
  additional_info: z.string().optional().nullable(),
  invoice_prefix: z
    .string()
    .min(1, "Le prefixe est requis")
    .regex(/^[A-Za-z0-9-]+$/, "Lettres, chiffres et tirets uniquement"),
  date_format: z.string().min(1),
  language: z.string().min(1),
  theme: z.enum(["light", "dark"]),
  invoice_template: z.enum(["classic", "modern", "minimal"]),
});

export type SettingsFormValues = z.infer<typeof settingsSchema>;

export const tenantSchema = z.object({
  first_name: z.string().min(1, "Le prenom est requis"),
  last_name: z.string().min(1, "Le nom est requis"),
  phone: z.string().min(6, "Le telephone est requis"),
  email: z
    .string()
    .email("Adresse email invalide")
    .optional()
    .or(z.literal(""))
    .nullable(),
  address: z.string().min(3, "L'adresse est requise"),
  id_number: z.string().optional().nullable(),
  profession: z.string().optional().nullable(),
  notes: z.string().optional().nullable(),
});

export type TenantFormValues = z.infer<typeof tenantSchema>;

export const invoiceSchema = z
  .object({
    tenant_id: z.coerce.number().int().positive("Selectionnez un locataire"),
    property_address: z.string().min(3, "L'adresse du bien est requise"),
    description: z.string().optional().nullable(),
    billing_month: z.coerce.number().int().min(1).max(12),
    billing_year: z.coerce.number().int().min(2000).max(2100),
    issue_date: z.string().min(1, "La date est requise"),
    due_date: z.string().min(1, "La date d'echeance est requise"),
    rent_amount: z.coerce.number().nonnegative("Le loyer doit etre positif"),
    water_charge: z.coerce.number().nonnegative().default(0),
    electricity_charge: z.coerce.number().nonnegative().default(0),
    other_charges: z.coerce.number().nonnegative().default(0),
    discount: z.coerce.number().nonnegative().default(0),
    amount_paid: z.coerce.number().nonnegative().default(0),
    payment_method: z.enum(["cash", "bank_transfer", "mobile_money", "check", "other"]),
    observations: z.string().optional().nullable(),
  })
  .refine(
    (data) => {
      const total =
        data.rent_amount +
        data.water_charge +
        data.electricity_charge +
        data.other_charges -
        data.discount;
      return total >= 0;
    },
    { message: "Le total ne peut pas etre negatif", path: ["discount"] }
  );

export type InvoiceFormValues = z.infer<typeof invoiceSchema>;
