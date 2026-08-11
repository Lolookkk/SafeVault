use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write,Read};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum VaultError {
    #[error("Erreur lors de la manipulation de fichiers : {0}")]
    FileError(#[from] std::io::Error),

    #[error("Erreur lors de la conversion de types : {0}")]
    ConversionError(#[from] serde_json::Error),

    #[error("Erreur lors des appels aux fonctions de crypto : {0}")]
    CryptographyError(#[from] crate::crypto::CryptoError),
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct PasswordEntry {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
}

/// Représente l'ensemble du coffre-fort contenant tous les comptes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vault {
    pub entries: Vec<PasswordEntry>,
    pub salt: [u8; 16], // Ajout du champ 'salt' pour stocker le sel utilisé pour la dérivation de clé
}

impl Vault {
    /// Crée un coffre-fort totalement vide., constructeur, pour l appeler ailleurs on écrit Vault::new()
    pub fn new() -> Self {
        Vault {
            //Initialise le champ 'entries' avec un vecteur vide ---
            entries: vec![],
            salt: [0u8; 16], // Initialisation du sel à zéro, il sera généré lors de la création du coffre
        }
    }

    /// Sauvegarde le coffre-fort dans un fichier au format JSON.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P, password: &str, salt : &[u8; 16]) -> Result<(), VaultError> {
        // 1. On convertit toute la structure self (le coffre) en une chaîne de texte JSON
        // La bibliothèque serde_json possède une fonction 'to_string' pour cela
        let json_text = serde_json::to_string(&self)?;

        let key= crate::crypto::derive_key(password, salt)?;
        
        let encrypted_data = crate::crypto::encrypt_data(json_text.as_bytes(),&key)?;

        // 2. On crée ou ouvre le fichier sur le disque dur
        let mut file = File::create(path)?;

        // 3. On écrit le texte JSON et le sel/salt dans le fichier
        file.write_all(salt)?;
        // Applique la méthode d'écriture sur 'file' en lui passant les octets de 'json_text' &[u8] ---
        file.write_all(&encrypted_data)?;

        Ok(())
    }

    /// Charge un coffre-fort à partir d'un fichier JSON.
    pub fn load_from_file<P: AsRef<Path>>(path: P,password: &str) -> Result<Self, VaultError> {
        // 1. On ouvre le fichier existant
        let mut file = File::open(path)?;

        // 2. On doit lire tout le contenu du fichier et le mettre dans un vecteur d'octets (données brutes)
        let mut encrypted_data: Vec<u8> = Vec::new();
        // 1. On prépare une boîte fixe de 16 octets pour accueillir le sel
        let mut salt = [0u8; 16];
        
        // 2. On remplit cette boîte avec les 16 premiers octets du fichier
        file.read_exact(&mut salt)?;
        
        let key= crate::crypto::derive_key(password, &salt)?;

        // --- TROU 5 : Applique la méthode sur 'file' pour lire tout son contenu et l'injecter dans '&mut json_text' ---
        file.read_to_end(&mut encrypted_data)?;

        
        let decrypted_data = crate::crypto::decrypt_data(&encrypted_data, &key)?;

        // 3. On reconstruit notre structure Vault à partir du texte JSON
        let vault: Vault = serde_json::from_slice(&decrypted_data)?;

        Ok(vault)
    }

    /// Supprime une entrée par sa ID.
    pub fn supprimer_entree(&mut self, id: String) {
        self.entries.retain(|e| e.id != id);
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_vault_new_and_save() {
        // 1. Préparation des données de test
        let mut vault = Vault::new();
        
        // On vérifie que le constructeur crée bien un coffre vide
        assert!(vault.entries.is_empty());

        let entry = PasswordEntry {
            id: "1".to_string(),
            title: "Netflix".to_string(),
            username: "user@email.com".to_string(),
            password: "super_password_123".to_string(),
            url: Some("https://netflix.com".to_string()),
        };
        vault.entries.push(entry);

        // 2. Configuration du chiffrement
        let test_path = "test_vault.enc";
        let password = "mon_master_password";
        let salt = [42u8; 16]; // Un sel fixe pour le test

        // 3. Exécution de la sauvegarde
        let result = vault.save_to_file(test_path, password, &salt);
        
        // On vérifie que la sauvegarde s'est bien déroulée
        assert!(result.is_ok());

        // 4. Vérification que le fichier existe et n'est pas vide
        let metadata = fs::metadata(test_path);
        assert!(metadata.is_ok());
        assert!(metadata.unwrap().len() > 16); // Plus grand que 16 octets (sel + données)

        // 5. Nettoyage après le test
        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_vault_load() {
        let mut vault_a_sauver = Vault::new();
            let entry = PasswordEntry {
            id: "1".to_string(),
            title: "Netflix".to_string(),
            username: "user@email.com".to_string(),
            password: "super_password_123".to_string(),
            url: Some("https://netflix.com".to_string()),
        };
        vault_a_sauver.entries.push(entry);

        // 2. Configuration du chiffrement
        let test_path = "test_vault.enc";
        let password = "mon_master_password";
        let salt = [42u8; 16];

        vault_a_sauver.save_to_file(test_path, password, &salt).unwrap();

        // 3. Exécution du téléchargement
        let vault = Vault::load_from_file(test_path, password).unwrap();
        assert!(vault.entries.is_empty() == false);

        //Vérification que le coffre rechargé contient bien la même entrée
        vault.entries.iter().for_each(|entry| {
            assert_eq!(entry.id,"1");
            assert_eq!(entry.title, "Netflix");
            assert_eq!(entry.username, "user@email.com");
            assert_eq!(entry.password, "super_password_123");
            assert_eq!(entry.url.as_ref().unwrap(), "https://netflix.com");
        });

        // 4. Nettoyage après le test
        let _ = fs::remove_file(test_path);
    }
}