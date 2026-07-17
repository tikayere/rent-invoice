import { open, save, ask, message } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";

/** Opens a native "select image" dialog and returns the chosen absolute path, or null if cancelled. */
export async function pickImageFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg"] }],
  });
  return typeof result === "string" ? result : null;
}

/** Opens a native "save file" dialog for a PDF invoice and returns the destination path. */
export async function pickPdfSaveTarget(defaultName: string): Promise<string | null> {
  const result = await save({
    defaultPath: defaultName,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  return result ?? null;
}

/** Opens a native "save file" dialog for a CSV export. */
export async function pickCsvSaveTarget(defaultName: string): Promise<string | null> {
  const result = await save({
    defaultPath: defaultName,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  return result ?? null;
}

/** Opens a native "save file" dialog for a SQLite database backup. */
export async function pickBackupSaveTarget(defaultName: string): Promise<string | null> {
  const result = await save({
    defaultPath: defaultName,
    filters: [{ name: "Base de donnees SQLite", extensions: ["db", "sqlite"] }],
  });
  return result ?? null;
}

/** Opens a native "open file" dialog to select a database backup to restore. */
export async function pickBackupToRestore(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Base de donnees SQLite", extensions: ["db", "sqlite"] }],
  });
  return typeof result === "string" ? result : null;
}

export async function confirmDestructive(text: string, title = "Confirmation"): Promise<boolean> {
  return ask(text, { title, kind: "warning" });
}

export async function notify(text: string, title = "Information"): Promise<void> {
  await message(text, { title, kind: "info" });
}

/** Converts a local filesystem path (e.g. a logo stored on disk) into a src usable by <img>. */
export function toAssetSrc(path: string | null | undefined): string | undefined {
  if (!path) return undefined;
  return convertFileSrc(path);
}
