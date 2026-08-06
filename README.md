# Gestion des Loyers — Application Desktop de Facturation

Application desktop 100% locale (Tauri v2 + Rust + React/TypeScript) pour
bailleurs immobiliers : gestion des locataires, generation de factures de
loyer en moins d'une minute, export PDF professionnel, historique, sauvegarde
et restauration de la base de donnees. Aucune connexion Internet requise
apres installation ; aucune donnee ne quitte l'ordinateur.

## Stack technique

- **Frontend** : React 18, TypeScript, Vite, TailwindCSS, React Hook Form + Zod, React Router
- **Backend** : Rust, Tauri v2, rusqlite (SQLite, mode bundled — aucune dependance systeme), printpdf 0.9 (generation PDF native, sans navigateur headless)
- **Base de donnees** : SQLite locale (fichier stocke dans le dossier de donnees de l'application), migrations automatiques au demarrage
- **Polices PDF** : DejaVu Sans (Regular + Bold) embarquees dans le binaire — rendu identique sur toutes les machines, accents francais geres nativement

## Prerequis

1. **Node.js** 18+ et npm
2. **Rust** (stable, recent — installez via [rustup.rs](https://rustup.rs), la version fournie par certains gestionnaires de paquets Linux est souvent trop ancienne pour les dependances actuelles de Tauri v2)
3. **Dependances systeme Tauri** (Linux uniquement — inutile sur Windows/macOS) :
   ```bash
   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev build-essential curl wget file libssl-dev pkg-config
   ```
   Sur Windows, installez les [Outils de developpement C++ Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) et WebView2 (preinstalle sur Windows 10/11 recents).

## Installation

```bash
npm install
```

## Icones de l'application

Un icone source (`app-icon.png`, 1024x1024) est fourni a la racine du projet.
Avant le premier `tauri build` (ou `tauri dev` si vous voulez voir la bonne
icone dans la barre des taches), generez le jeu complet d'icones (.ico, .icns,
tailles PNG) avec la CLI Tauri :

```bash
npx tauri icon app-icon.png
```

Cela remplit automatiquement `src-tauri/icons/`. Remplacez `app-icon.png` par
votre propre logo si besoin avant de lancer cette commande.

## Lancer en developpement

```bash
npm run tauri dev
```

## Compiler pour la production

```bash
npm run tauri build
```

Cela genere, selon la plateforme :
- Windows : un installeur `.msi` et `.exe` (NSIS) dans `src-tauri/target/release/bundle/`
- Linux : `.deb` et `.AppImage`
- macOS : `.app` et `.dmg`

## Compiler pour macOS (localement, sur un Mac)

Le build desktop macOS suit le meme principe que ci-dessus
(`npm run tauri build`), avec quelques particularites propres a macOS
detaillees ici : produire un binaire universel (Intel + Apple Silicon) et
contourner l'avertissement Gatekeeper puisque l'app n'est pas signee par un
certificat Apple Developer payant. (Un workflow CI signe et notarise existe
dans `.github/workflows-on-hold/build-macos.yml`, en pause pour l'instant —
a reactiver plus tard si vous voulez distribuer sans avertissement et sans
repasser par un Mac a chaque fois.)

### Prerequis (sur le Mac)

Node.js et Rust comme decrit dans la section [Prerequis](#prerequis)
ci-dessus, plus les deux cibles Rust pour produire un binaire universel :

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

### Etapes

1. Cloner le depot et installer les dependances :
   ```bash
   git clone <url-du-depot>
   cd rent-invoice
   npm install
   ```
2. Compiler en binaire universel (fonctionne aussi bien sur Mac Intel
   qu'Apple Silicon) :
   ```bash
   npm run tauri build -- --target universal-apple-darwin
   ```
   Le `.app` et le `.dmg` sont generes dans
   `src-tauri/target/universal-apple-darwin/release/bundle/`. Pour un build
   plus rapide limite a l'architecture de la machine courante, omettez
   `--target universal-apple-darwin` (build alors dans
   `src-tauri/target/release/bundle/`).
3. A la premiere ouverture, macOS affiche un avertissement "developpeur non
   identifie" (l'app n'est pas signee/notarisee). Pour l'ouvrir malgre tout :
   **clic droit sur l'app > Ouvrir > confirmer "Ouvrir"** dans la boite de
   dialogue (une seule fois ; les lancements suivants se font normalement en
   double-cliquant).

## Compiler pour iOS (localement, sur un Mac)

La compilation iOS se fait en local sur un Mac plutot que via une CI : cela
evite d'avoir a gerer un certificat de distribution, un profil de
provisioning ou des cles API — Xcode gere la signature automatiquement avec
votre Apple ID. (Le workflow GitHub Actions correspondant existe toujours
dans `.github/workflows-on-hold/build-ios.yml` mais est en pause ; il peut
etre reactive plus tard si besoin de distribuer sans repasser par un Mac a
chaque fois.)

### Prerequis (sur le Mac)

1. **Xcode** (App Store), puis les outils en ligne de commande :
   ```bash
   xcode-select --install
   ```
2. Dans Xcode : **Settings > Accounts**, connectez-vous avec un Apple ID.
   Un compte gratuit suffit pour installer l'app sur vos propres
   iPhone/iPad ; un compte payant (Apple Developer Program, 99$/an) n'est
   necessaire que pour TestFlight/l'App Store.
3. **Node.js** 18+ (`brew install node` ou [nodejs.org](https://nodejs.org))
4. **Rust** + cibles iOS :
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
   ```

### Etapes

1. Cloner le depot et installer les dependances :
   ```bash
   git clone <url-du-depot>
   cd rent-invoice
   npm install
   ```
2. Initialiser le projet Xcode :
   ```bash
   npm run tauri ios init
   ```
3. Deux options pour lancer l'app sur un iPhone/iPad connecte en USB :
   - **Test rapide avec hot-reload** :
     ```bash
     npm run tauri ios dev
     ```
     La premiere fois, Tauri ouvre Xcode : dans l'onglet
     **Signing & Capabilities** du target `rent-invoice-manager_iOS`,
     selectionnez votre Team (votre "Personal Team" si compte gratuit) avec
     **Automatically manage signing** coche, puis relancez la commande.
   - **Build et installation manuelle depuis Xcode** :
     ```bash
     npm run tauri ios build -- --open
     ```
     Cela ouvre le projet dans Xcode : choisissez votre appareil comme
     destination (en haut, a cote du bouton Run), verifiez la Team dans
     Signing & Capabilities, puis **Product > Run** (Cmd+R) pour compiler et
     installer directement sur l'appareil.
4. A la premiere installation, faire confiance au profil de developpeur sur
   l'iPhone : **Reglages > General > VPN et gestion de l'appareil >
   \[votre Apple ID\] > Faire confiance**.

> Avec un compte Apple Developer gratuit, l'app installee expire au bout de
> 7 jours (relancez simplement Run depuis Xcode pour la reinstaller). Avec
> un compte payant, les builds signes durent 1 an.

## Structure du projet

```
src/                        Frontend React/TypeScript
  components/ui/            Composants d'interface reutilisables (Button, Card, Input, Modal...)
  components/layout/        Sidebar, Header
  pages/                    Dashboard, Tenants, InvoiceForm, History, Settings
  hooks/                    useSettings, useTenants, useDashboard
  services/                 Wrappers types autour de `invoke()` et des dialogues natifs
  types/                    Types partages avec le backend Rust
  utils/                    Formatage (devise, dates), schemas de validation Zod

src-tauri/                  Backend Rust
  src/commands/             Commandes Tauri exposees au frontend (une par domaine)
  src/database/             Connexion SQLite + migrations automatiques
  src/models/               Structures de donnees (Settings, Tenant, Invoice...)
  src/services/             Logique metier (numerotation, sauvegarde, export CSV)
  src/pdf/                  Generation du PDF de facture (printpdf) + polices embarquees
  assets/fonts/             Polices DejaVu Sans embarquees dans le binaire
  capabilities/             Permissions Tauri v2 (ACL)
```

## Fonctionnalites

- **Tableau de bord** : nombre de factures, de locataires, derniere facture, montant encaisse ce mois-ci, reste a payer total
- **Locataires** : creation, modification, suppression (bloquee si des factures existent), recherche
- **Factures** : formulaire avec calcul automatique (loyer + eau + electricite + autres frais - remise = total ; total - paye = reste), numerotation automatique configurable (prefixe-annee-sequence, ex. `LOY-2026-000001`), statut automatique (paye / partiellement paye / non paye)
- **PDF** : genere nativement en Rust (aucun navigateur headless requis), avec logo, informations du bailleur, tableau des charges, totaux, signature, pied de page — pret pour impression A4
- **Historique** : recherche, filtres (statut, annee), tri, pagination, reimpression, suppression, export CSV
- **Parametres** : informations du bailleur, logo, signature, devise, format de date, prefixe de facturation, theme clair/sombre
- **Sauvegarde** : export/import du fichier SQLite complet, avec sauvegarde de securite automatique avant toute restauration

## Notes de conception

- Toute la logique metier (calculs, numerotation, validation) vit cote Rust ; le frontend n'est qu'une couche de presentation qui reflete l'etat retourne par le backend, evitant toute divergence entre ce qui est affiche et ce qui est enregistre.
- Les montants sont stockes et calcules en `f64` arrondis au centime ; aucune bibliotheque de devise externe n'est necessaire, le formatage d'affichage utilise `Intl.NumberFormat` cote frontend et un formatage simple cote PDF.
- Le schema de base de donnees suit la structure demandee (`settings`, `tenants`, `invoices`, `invoice_items`, `payments`) et est cree automatiquement au premier lancement ; `invoice_items` et `payments` sont deja peuples pour permettre des evolutions futures (paiements partiels multiples, factures a lignes dynamiques) sans migration de schema.
- Architecture prete pour l'internationalisation (le champ `language` existe dans les parametres) meme si seul le francais est actif aujourd'hui, et pour plusieurs bailleurs/biens/devises via extension du schema `settings`.
