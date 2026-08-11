use thiserror::Error;
use rand::RngCore;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Erreur lors de la dérivation de la clé (Argon2id)")]
    KeyDerivationError,

    #[error("Erreur lors du chiffrement des données (AES-GCM)")]
    EncryptionError,
    
    #[error("Erreur lors du déchiffrement : mot de passe maître incorrect ou données corrompues")]
    DecryptionError, 

    #[error("Erreur lors de la génération du mot de passe aléatoire")]
    PasswordGenerationError,
}

use argon2::{
    Argon2, Algorithm, Version, Params
};

pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}


pub fn derive_key(master_password: &str, salt: &[u8; 16]) -> Result<Vec<u8>, CryptoError> {
    let params = Params::new(65536, 3, 4, Some(32)).map_err(|_| CryptoError::KeyDerivationError)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut derived_key = vec![0u8; 32];

    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut derived_key)
        .map_err(|_| CryptoError::KeyDerivationError)?;
    Ok(derived_key)

}

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};



pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // 1. Initialisation du chiffreur avec la clé de 32 octets
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::EncryptionError)?;

    // 2. Génération d'un Nonce unique de 12 octets au hasard
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 3. Chiffrement des données
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|_| CryptoError::EncryptionError)?;

    // 4. Assemblage du Nonce et du texte chiffré
    let mut encrypted_packet = nonce_bytes.to_vec();
    encrypted_packet.extend_from_slice(&ciphertext);

    Ok(encrypted_packet)
}


/// Déchiffre des données protégées par l'AES-256-GCM.
/// Reçoit le paquet combiné (Nonce + Texte chiffré) et la clé de 32 octets.
pub fn decrypt_data(encrypted_packet: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // Vérification de sécurité : le paquet doit au moins contenir les 12 octets du Nonce
    if encrypted_packet.len() < 12 {
        return Err(CryptoError::DecryptionError);
    }

    // 1. On sépare le Nonce et le texte chiffré
    // En Rust, on peut "découper" un tableau avec les indices [début..fin]
    let (nonce_bytes, ciphertext) = encrypted_packet.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 2. Initialisation du chiffreur (la machine)
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::DecryptionError)?;

    // 3. Déchiffrement des données
    let decrypted_data = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionError)?;

    Ok(decrypted_data)
}

use rand::Rng; // Pour pouvoir utiliser gen_range

/// Génère un mot de passe aléatoire et hautement sécurisé.
pub fn generate_password(
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_special: bool,
) -> Result<String, CryptoError> {
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let numbers = "0123456789";
    let special = "!@#$%^&*()_+-=[]{}|;:,.<>?";

    // 1. On crée une String vide qui va contenir tous les caractères autorisés
    let mut pool = String::new();
    
    // --- TROU 1 : Ajoute les bonnes chaînes dans `pool` selon les choix de l'utilisateur ---
    if use_uppercase {
        pool.push_str(uppercase);
    }
    if use_lowercase {
        pool.push_str(lowercase);
    }
    if use_numbers   {
        pool.push_str(numbers);
    }
    if use_special   {
        pool.push_str(special);
    }

    // Sécurité : Si la banque est vide, on s'arrête
    if pool.is_empty() {
        return Err(CryptoError::PasswordGenerationError);
    }

    // On transforme la réserve en un vecteur de caractères pour pouvoir piocher dedans par index
    let pool_chars: Vec<char> = pool.chars().collect();
    
    // 2. Initialisation de notre mot de passe final (vide au départ)
    let mut password = String::new();// (TROU 2 : Crée une String vide)

    // 3. Boucle pour piocher les caractères un par un
    // --- TROU 3 : Écris la boucle qui doit tourner "length" fois ---
    for _ in 0..length {
        // On pioche un index au hasard entre 0 et la taille du vecteur
        let random_index = OsRng.gen_range(0..pool_chars.len());
        
        // --- TROU 4 : Ajoute le caractère pioché à notre variable `password` ---
        password.push(pool_chars[random_index]);
    }

    Ok(password)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_success() {
        let password = "MonMotDePasseMaitre123!";
        let salt = b"UnSelDe16OctetsM"; // Le 'b' devant signifie "tableau de bytes (octets)"

        // 1. On lance la dérivation
        let result = derive_key(password, salt);

        // 2. On vérifie que la fonction n'a pas renvoyé d'erreur
        assert!(result.is_ok());

        // 3. On récupère la clé
        let key = result.unwrap();

        // 4. On vérifie que la clé fait bien exactement 32 octets (256 bits)
        assert_eq!(key.len(), 32);

        // 5. Bonus : On vérifie que si on change un seul caractère du mot de passe, la clé est différente
        let alternative_password = "MonMotDePasseMaitre123?";
        let alternative_key = derive_key(alternative_password, salt).unwrap();
        assert_ne!(key, alternative_key);
        
        println!("Clé générée avec succès : {:?}", key);
    }

    #[test]
    fn test_encrypt_data_success() {
        let password = "mon_super_mot_de_passe";
        let salt = [0u8; 16]; 
        let key = derive_key(password, &salt).unwrap();

        let data = b"donnees_secretes_du_coffre_fort";
//         let data_text = "Les données à chiffrer";
// let data = data_text.as_bytes(); // Transforme le texte avec accents en octets UTF-8
        let packet1 = encrypt_data(data, &key).unwrap();
        let packet2 = encrypt_data(data, &key).unwrap();
        
        assert_ne!(packet1, packet2);

        // Vérification 2 : La taille du paquet doit être égale à la taille du Nonce (12) + taille du texte chiffré
        // L'AES-GCM ne change pas la taille des données, mais ajoute un "Tag" d'authentification de 16 octets.
        // Donc taille attendue = 12 (Nonce) + data.len() + 16 (Tag)
        let expected_packet_size = 12 + data.len() + 16;
        assert_eq!(packet1.len(), expected_packet_size);
    }


    #[test]
    fn test_encrypt_and_decrypt() {
        let password = "mon_super_mot_de_passe";
        let salt = [0u8; 16];
        let key = derive_key(password, &salt).unwrap();
        let data = b"donnees_secretes";

        // 1. On chiffre
        let packet = encrypt_data(data, &key).unwrap();

        // 2. On déchiffre
        let decrypted = decrypt_data(&packet, &key).unwrap();

        // 3. Vérification : les données déchiffrées doivent être IDENTIQUES aux données de départ
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_generate_password() {
        // On demande un mot de passe de 16 caractères, uniquement avec des chiffres
        let pwd = generate_password(16, false, false, true, false).unwrap();
        
        // Vérification 1 : La longueur est correcte
        assert_eq!(pwd.len(), 16);
        
        // Vérification 2 : Il ne contient que des chiffres
        // La méthode .chars().all(|c| c.is_ascii_digit()) renvoie true si TOUS les caractères sont des chiffres
        assert!(pwd.chars().all(|c| c.is_ascii_digit()));
    }
}


