# 🔐 SafeVault — Gestionnaire de Mots de Passe en Rust

Un gestionnaire de mots de passe sécurisé, **100% local** et performant développé en **Rust (édition 2024)**. Le projet combine une interface graphique bureau (GUI) et un serveur HTTP local (Axum) permettant le remplissage automatique via une extension navigateur.

**Nom du crate** : `safevault` — **Version** : `0.1.0`

---

## ✨ Fonctionnalités (implémentées à 100%)

- 🔒 **Chiffrement fort AES-256-GCM** : chiffrement authentifié avec tag d'intégrité (16 octets) + nonce unique par chiffrement.
- 🔑 **Dérivation de clé Argon2id** : KDF sécurisé contre les attaques par force brute (m=64 MiB, t=3 itérations, p=4 parallèles, sortie 256 bits).
- 🏠 **Interface graphique bureau (egui/eframe)** :
  - Création de coffre avec **double saisie** du mot de passe maître + indicateur visuel de correspondance
  - Déverrouillage / verrouillage instantané
  - Ajout, affichage, copie et **suppression** d'entrées
  - Générateur de mots de passe aléatoires (configurable : majuscules, minuscules, chiffres, spéciaux)
  - Affichage / masquage des mots de passe par entrée
- 🌐 **Serveur HTTP local (Axum)** :
  - Endpoint `GET /api/credentials?domain=xxx` sur le port **127.0.0.1:8765**
  - CORS ouvert (extension navigateur) via `tower-http`
  - Recherche floue dans l'URL ou le titre de l'entrée
  - Réponse `Json(None)` si coffre verrouillé ou aucune correspondance
- 🧵 **État partagé thread-safe** : `Arc<Mutex<AppStateServeur>>` synchronisé entre GUI (egui) et serveur HTTP (tokio).

---

## 🛠️ Stack Technique (Cargo.toml exact)

| Catégorie | Crate | Version | Rôle exact dans le code |
| :--- | :--- | :--- | :--- |
| **Langage** | Rust | 🦀 Edition 2024 | |
| **GUI** | `eframe` | `0.26` | Framework d'exécution egui natif |
| | `egui` | `0.26` | Bibliothèque d'interface graphique immédiate |
| | `uuid` | `1.23.4` (`v4`) | Génération des IDs uniques des entrées |
| **Cryptographie** | `argon2` | `0.5` (`std`) | KDF Argon2id — dérivation de la clé 256-bit |
| | `aes-gcm` | `0.10` | Chiffrement symétrique authentifié AES-256-GCM |
| | `rand` | `0.8` | Génération CSPRNG : salts, nonces, passwords |
| **Sérialisation** | `serde` | `1.0.228` (`derive`) | Sérialisation JSON des structures Vault & PasswordEntry |
| | `serde_json` | `1.0.150` | (Dé)sérialisation JSON vers/depuis le fichier chiffré |
| **Erreurs** | `thiserror` | `1.0` | Types d'erreurs custom : `CryptoError`, `VaultError` |
| **Serveur HTTP** | `axum` | `0.7` | Router et handler async |
| | `tokio` | `1.0` (`full`) | Runtime async dans un thread dédié |
| | `tower-http` | `0.5` (`cors`) | Middleware CORS Any/Any/Any |
| **Concurrence** | `std::sync` | — | `Arc<Mutex<T>>` pour partager `SafeState` |

---

## 📁 Structure du Projet (exacte)

```
safevault/
├── Cargo.toml
├── vault.json               ← Fichier du coffre (16 octets salt + données AES-GCM)
└── src/
    ├── main.rs              ← Point d'entrée : init SafeState, lance GUI + serveur
    ├── crypto.rs            ← Argon2id, AES-256-GCM, generate_password() (+ tests unitaires)
    ├── storage.rs           ← Vault, PasswordEntry, load/save dans vault.json (+ tests unitaires)
    ├── server.rs            ← start_server(), route /api/credentials, SafeState = Arc<Mutex<AppStateServeur>>
    └── gui.rs               ← SafeVaultApp (eframe::App), états Verrouille/Deverrouille
```

---

## 🧠 Architecture & État Partagé

```
┌───────────────────────────────────┐     ┌──────────────────────────────────────────┐
│  Interface GUI (thread principal) │     │  Serveur HTTP Axum (thread + tokio)      │
│  (egui via eframe)                │     │  127.0.0.1:8765                          │
└──────────────┬────────────────────┘     └──────────────┬───────────────────────────┘
               │                                         │
               │  write (déverrouillage / CRUD)          │  read (requête ?domain=)
               ▼                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  SafeState = Arc<Mutex<AppStateServeur>>                                             │
│  ├── coffre_ouvert: Some(Vec<PasswordEntry>)   ← GUI le remplit / le vide           │
│  └── coffre_ouvert: None                        ← Serveur renvoie Json(None)         │
└──────────────────────────────────────────────────────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  Fichier vault.json                                                                   │
│  ├── 16 octets : salt aléatoire (généré à la création du coffre)                     │
│  └── N octets  : AES-256-GCM packet = [12 nonce | ciphertext | 16 tag auth]          │
│                      → chiffré avec clé = Argon2id(master_pwd, salt)                 │
│                      → contenu en clair = JSON(Vault { entries, salt })              │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

**Règles d'accès garanties par Mutex :**
- Le coffre ne peut jamais être lu pendant une modification
- Au verrouillage : `coffre_ouvert = None` (aucune donnée en clair accessible au serveur)

---

## 🔐 Détails Cryptographiques (correspondance code exact)

### Dérivation de clé — `crypto::derive_key()`
```
Argon2id
  ├── Algorithm : Argon2id
  ├── Version   : V0x13
  ├── Params    : m_cost = 65536 (KiB), t_cost = 3, parallelism = 4
  ├── Sortie    : 32 octets (AES-256)
  └── Salt      : 16 octets, stockés en clair en tête de vault.json
```

### Chiffrement — `crypto::encrypt_data()`
```
AES-256-GCM
  ├── Nonce     : 12 octets aléatoires (OsRng)
  ├── Tag auth  : 16 octets (auto par AES-GCM)
  ├── Packet    : [nonce (12)] + [ciphertext (N)] + [tag (16)]
  └── Note      : 2 chiffrements du même plaintext → 2 packets différents
```

### Format stockage `vault.json`
```
<---- 16 octets salt ----><---- paquet AES-GCM ------------------------>
[  salt pour Argon2id   ][ nonce (12) | JSON(Vault) chiffré | tag (16) ]
```

---

## 🌐 API HTTP (serveur.rs — 127.0.0.1:8765)

### Endpoint
```
GET /api/credentials?domain=exemple.com
```

### Paramètre
| Nom | Type | Description |
| :--- | :--- | :--- |
| `domain` | `String` (query) | Domaine à chercher dans URL ou titre des entrées (insensible à la casse) |

### Réponse (JSON)

| Cas | Corps HTTP |
| :--- | :--- |
| **Coffre déverrouillé + correspondance trouvée** | `{ "username": "...", "password": "..." }` |
| **Coffre verrouillé OU aucune correspondance** | `null` |

### Exemple avec `curl`
```bash
# Coffre déverrouillé, contient une entrée "Netflix"
curl "http://127.0.0.1:8765/api/credentials?domain=netflix"
# → {"username":"johndoe","password":"SuperSecret123!"}

# Coffre verrouillé
curl "http://127.0.0.1:8765/api/credentials?domain=netflix"
# → null
```

---

## 🚀 Installation & Lancement

### Prérequis
Outilchain **Rust stable** (édition 2024) — dispo via [rustup.rs](https://rustup.rs).

```bash
rustc --version   # ≥ 1.85 (edition 2024)
cargo --version
```

### Démarrage rapide
```bash
# Compiler + lancer en mode développement
cargo run

# Compiler + lancer en mode release (optimisé)
cargo run --release

# Seulement compiler (sans lancer)
cargo build          # dev
cargo build --release
```

### Lancer les tests unitaires
```bash
cargo test
# → 5 tests :
#   crypto::tests::test_derive_key_success
#   crypto::tests::test_encrypt_data_success
#   crypto::tests::test_encrypt_and_decrypt
#   crypto::tests::test_generate_password
#   storage::tests::test_vault_new_and_save
#   storage::tests::test_vault_load
```

### Vérifier sans compiler
```bash
cargo check --message-format=short
```

---

## 🎯 Workflow de l'application

1. **Premier lancement** (pas de `vault.json`) :
   - Bouton « Créer un nouveau coffre-fort »
   - Saisir **2 fois** le mot de passe maître (indicateur vert/rouge)
   - Génération auto d'un salt 16 octets + création `vault.json` chiffré

2. **Lancements suivants** (`vault.json` existe) :
   - Saisir le mot de passe maître → bouton « Se connecter »
   - Appel interne : `Vault::load_from_file("vault.json", pwd)` → `derive_key()` → `decrypt_data()`

3. **Coffre déverrouillé** :
   - Ajouter des entrées (titre, username, password, URL)
   - Générer des MDP aléatoires
   - Copier / afficher / masquer / supprimer des entrées
   - Chaque modification est **automatiquement persistée** dans `vault.json`
   - Le `server_state` est synchronisé à chaque CRUD (disponible à l'extension)

4. **Verrouillage** :
   - Bouton « Verrouiller »
   - `server_state.coffre_ouvert = None` → le serveur renvoie `null` immédiatement

---

## 🧪 Tests Unitaires Inclus

| Module | Test | Vérifie |
| :--- | :--- | :--- |
| `crypto.rs` | `test_derive_key_success` | Clé = 32 octets ; 2 MDP similaires mais différents → 2 clés différentes |
| `crypto.rs` | `test_encrypt_data_success` | 2 chiffrements identiques → packets différents (nonce unique) ; taille = 12 + N + 16 |
| `crypto.rs` | `test_encrypt_and_decrypt` | Round-trip : `decrypt(encrypt(data)) == data` |
| `crypto.rs` | `test_generate_password` | Longueur respectée ; filtrage charset |
| `storage.rs` | `test_vault_new_and_save` | `Vault::new()`, ajout entrée, `save_to_file()`, fichier non vide |
| `storage.rs` | `test_vault_load` | `save` puis `load` → champs identiques (round-trip fichier) |

---

## 🛣️ Perspectives d'Évolution (non implémentées)

- [ ] Modification du mot de passe maître avec re-chiffrement complet
- [ ] Édition d'une entrée existante (GUI + CRUD manquant)
- [ ] Vidage automatique du presse-papier après TTL
- [ ] Indicateur de force du mot de passe maître (zxcvbn)
- [ ] Recherche / filtre des entrées dans le GUI
- [ ] TOTP (double authentification) pour certaines entrées
- [ ] Protection du fichier local (sauvegardes automatiques chiffrées & gestion des permissions système pour éviter la suppression accidentelle)
