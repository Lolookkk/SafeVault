use crate::storage::{PasswordEntry, Vault};
use crate::server::SafeState;
use rand::RngCore;

enum AppState {
    Verrouille,
    Deverrouille(Vault,String),
}

pub struct SafeVaultApp {
    state: AppState,
    pub server_state: SafeState,
    password_input: String,
    verification_password_input: String,
    mode_creation: bool,
    erreur: Option<crate::storage::VaultError>,
    add_title: String,
    add_username: String,
    add_password: String,
    show_create_password: bool,
    show_add_password: bool,
    add_url: String,
    password_visible_id: Option<String>,
}

impl SafeVaultApp {
    pub fn new(server_state: SafeState) -> Self {
        Self {
            state: AppState::Verrouille,
            server_state: server_state,
            password_input: String::new(),
            verification_password_input: String::new(),
            mode_creation: false,
            erreur: None,
            add_title: String::new(),
            add_username: String::new(),
            add_password: String::new(),
            show_create_password: false,
            show_add_password: false,
            add_url: String::new(),
            password_visible_id: None,
        }
    }
}

impl eframe::App for SafeVaultApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        match self.state {
            AppState::Verrouille => { 
                let mon_cadre = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(254, 243, 231))
                    .inner_margin(30.0);
                egui::CentralPanel::default().frame(mon_cadre).show(ctx, |ui| {
                    if !self.mode_creation {
                        let layout = egui::Layout::top_down(egui::Align::Center);
                        ui.with_layout(layout, |ui| {
                            ui.label("");
                                ui.group(|ui| {
                                    ui.set_width(300.0);
                                    ui.vertical_centered(|ui| {
                                        ui.label("Page de connexion");
                                        ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));
                                        if ui.button("Se connecter").clicked() {
                                            match Vault::load_from_file("vault.json", &self.password_input) {
                                                Ok(coffre) => {
                                                    let entries_copie = coffre.entries.clone();
                                                    self.server_state.lock().unwrap().coffre_ouvert = Some(entries_copie);
                                                    self.erreur = None;
                                                    self.state = AppState::Deverrouille(coffre, self.password_input.clone());
                                                },
                                                Err(e) => {
                                                    self.erreur = Some(e);
                                                }
                                            }
                                        }
                                        
                                        ui.label("Vous n'avez pas encore de coffre-fort ? Créez-en un nouveau !");
                                        if ui.button("Créer un nouveau coffre-fort").clicked() {
                                            self.password_input.clear();
                                            self.mode_creation = true;
                                        }
                                        
                                    });
                                    
                                });
                        
                        });
                        
                    }
                    else {
                        if std::path::Path::new("vault.json").exists() {
                            ui.label("Un coffre-fort existe déjà.");
                            if ui.button("Retour").clicked() {
                                self.mode_creation = false;
                            }
                        } else {
                            ui.label("Entrez un mot de passe maître :");
                            ui.add(egui::TextEdit::singleline(&mut self.password_input)
                            .password(!self.show_create_password));
                            ui.label("Entrez à nouveau votre mot de passe :");
                            ui.add(egui::TextEdit::singleline(&mut self.verification_password_input)
                            .password(!self.show_create_password));
                            
                            
                            let libelle_bouton_creation = if self.show_create_password { "🙈 Masquer" } else { "👁 Afficher" };
                            if ui.button(libelle_bouton_creation).clicked() {
                                self.show_create_password = !self.show_create_password;
                            }
                            
                            // Vérification des états
                            let mdp_vides = self.password_input.is_empty() || self.verification_password_input.is_empty();
                            let mdp_identiques = self.password_input == self.verification_password_input;

                            if !mdp_vides {
                                if mdp_identiques {
                                    ui.colored_label(egui::Color32::GREEN, "Les mots de passe correspondent.");
                                } else {
                                    ui.colored_label(egui::Color32::RED, "Les mots de passe ne correspondent pas !");
                                }
                            }

                            if ui.button("Valider").clicked() {
                                if mdp_vides {
                                    self.erreur = Some(crate::storage::VaultError::FileError(
                                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Le mot de passe ne peut pas être vide")
                                    ));
                            } else if !mdp_identiques {
                                self.erreur = Some(crate::storage::VaultError::FileError(
                                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Les mots de passe ne correspondent pas")
                                ));
                            } else {
                                    let mut salt = [0u8; 16];
                                    rand::thread_rng().fill_bytes(&mut salt);
                                    let coffre = Vault::new();
                                    match coffre.save_to_file("vault.json", &self.password_input, &salt) {
                                        Ok(()) => {
                                            let entries_copie = coffre.entries.clone();
                                            self.server_state.lock().unwrap().coffre_ouvert = Some(entries_copie);
                                            self.erreur = None;
                                            self.mode_creation = false;
                                            self.state = AppState::Deverrouille(coffre, self.password_input.clone());
                                            self.password_input.clear();
                                        },
                                        Err(e) => {
                                            self.erreur = Some(e);
                                        }
                                    }
                                    
                                }
                            }
                            
                        }
                        


                        
                    }

                    // 🎯 L'affichage global de l'erreur, commun à tout l'écran verrouillé !
                    if let Some(ref err) = self.erreur {
                        let message_propre = match err {
                            crate::storage::VaultError::FileError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                                "Le fichier du coffre-fort n'existe pas encore."
                            }
                            crate::storage::VaultError::FileError(io_err) if io_err.kind() == std::io::ErrorKind::InvalidInput => {
                                "Le mot de passe ne peut pas être vide !"
                            }
                            crate::storage::VaultError::CryptographyError(_) => {
                                "Mot de passe incorrect ou données corrompues."
                            }
                            _ => "Une erreur technique est survenue.",
                        };

                        ui.colored_label(egui::Color32::RED, message_propre);
                    }
                });
            },
            AppState::Deverrouille(ref mut coffre, ref mp) => {
                let mut doit_verrouiller = false;
                let mut ids_a_supprimer: Vec<String> = Vec::new();
                let mon_cadre = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(254, 243, 231))
                    .inner_margin(30.0);

                
                egui::CentralPanel::default().frame(mon_cadre).show(ctx, |ui| {
                    ui.label("Coffre-fort Déverrouillé");
                    //Création d'un nouveau mdp
                    ui.group(|ui| {
                        ui.label("Ajouter un nouveau mot de passe :");
                        ui.add(egui::TextEdit::singleline(&mut self.add_title).hint_text("Titre"));
                        ui.add(egui::TextEdit::singleline(&mut self.add_username).hint_text("Nom d'utilisateur"));
                        ui.add(egui::TextEdit::singleline(&mut self.add_password).hint_text("Mot de passe").password(!self.show_add_password));
                        if ui.button("Générer un mot de passe aléatoire").clicked() {
                            self.add_password = crate::crypto::generate_password(12,true,true,true,true).unwrap();
                        }
                        let libelle_bouton = if self.show_add_password { "🙈 Masquer" } else { "👁 Afficher" };
                        if ui.button(libelle_bouton).clicked() {
                            self.show_add_password = !self.show_add_password;
                        }
                        ui.add(egui::TextEdit::singleline(&mut self.add_url).hint_text("URL"));
                        
                        if ui.button("Ajouter").clicked() {
                            match self.add_title.is_empty() || self.add_username.is_empty() || self.add_password.is_empty() {
                                true => {
                                    self.erreur = Some(crate::storage::VaultError::FileError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Tous les champs sauf l'URL doivent être remplis")));
                                },
                                false => {
                                    self.erreur = None;
                                }
                            }
                            if self.erreur.is_none() {
                                let new_entry = PasswordEntry {
                                id: uuid::Uuid::new_v4().to_string(),
                                title: self.add_title.clone(),
                                username: self.add_username.clone(),
                                password: self.add_password.clone(),
                                url: Some(self.add_url.clone()),
                                };

                                coffre.entries.push(new_entry);
                                match coffre.save_to_file("vault.json", mp, &coffre.salt) {
                                    Ok(()) => {
                                        let entries_copie = coffre.entries.clone();
                                        self.server_state.lock().unwrap().coffre_ouvert = Some(entries_copie);
                                        self.erreur = None;
                                    },
                                    Err(e) => {
                                        self.erreur = Some(e);
                                    }
                                }
                            
                                self.add_title.clear();
                                self.add_username.clear();
                                self.add_password.clear();
                                self.add_url.clear();
                                self.show_add_password = false;
                            }
                            
                            
                        }

                        // 🎯 L'affichage global de l'erreur, commun à tout l'écran déverrouillé !
                        if let Some(ref err) = self.erreur {
                            let message_propre = match err {
                                crate::storage::VaultError::FileError(io_err) if io_err.kind() == std::io::ErrorKind::InvalidInput => {
                                    "Vous n'avez pas rempli tout les champs obligatoires !"
                                }
                                crate::storage::VaultError::CryptographyError(_) => {
                                    "Données corrompues."
                                }
                                _ => "Une erreur technique est survenue.",
                            };
                            ui.colored_label(egui::Color32::RED, message_propre);
                        }

                    });
                    
                    //Affichage des mdp
                    for entree in &coffre.entries {
                        ui.push_id(&entree.id, |ui|{
                            egui::CollapsingHeader::new(&entree.title).show(ui, |ui| {
                                    ui.label(&entree.username);

                                    ui.horizontal(|ui| {
                                        let est_ce_le_mot_de_passe_visible = self.password_visible_id.as_deref() == Some(&entree.id);
                                        if est_ce_le_mot_de_passe_visible {
                                            ui.label(&entree.password);
                                        } else {
                                            ui.label("********");
                                        }
                                        if ui.button("Copier").clicked() {
                                            ui.output_mut(|o| o.copied_text = entree.password.clone());
                                        }
                                                            if est_ce_le_mot_de_passe_visible {
                                        if ui.button("🙈 Masquer").clicked() {
                                            self.password_visible_id = None;
                                        }
                                        } else {
                                            if ui.button("👁 Afficher").clicked() {
                                                self.password_visible_id = Some(entree.id.clone());
                                            }
                                        }
                                    });
                                    if let Some(url) = &entree.url {
                                        ui.hyperlink(url);
                                    }
                                    if ui.button("Supprimer").clicked() {
                                        ids_a_supprimer.push(entree.id.clone());
                                    }

                            });
                        });
                    }
                    if ui.button("Verrouiller").clicked() {
                        doit_verrouiller = true;
                    }
                    
                });

                if !ids_a_supprimer.is_empty() {
                    coffre.entries.retain(|e| !ids_a_supprimer.contains(&e.id));
                    if let Some(id_visible) = &self.password_visible_id {
                        if ids_a_supprimer.contains(id_visible) {
                            self.password_visible_id = None;
                        }
                    }
                    match coffre.save_to_file("vault.json", mp, &coffre.salt) {
                        Ok(()) => {
                            let entries_copie = coffre.entries.clone();
                            self.server_state.lock().unwrap().coffre_ouvert = Some(entries_copie);
                            self.erreur = None;
                        },
                        Err(e) => {
                            self.erreur = Some(e);
                        }
                    }
                }

                if doit_verrouiller {
                    self.server_state.lock().unwrap().coffre_ouvert = None;
                    self.password_input.clear();
                    self.verification_password_input.clear();
                    self.password_visible_id = None;
                    self.show_create_password = false;
                    self.show_add_password = false;
                    self.mode_creation = false;
                    self.erreur = None;
                    self.add_title.clear();
                    self.add_username.clear();
                    self.add_password.clear();
                    self.add_url.clear();
                    self.state = AppState::Verrouille;
                }
                

            },
        }
    }
}