use axum::{
    extract::{Query,State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use std::sync::{Arc, Mutex};
use crate::storage::PasswordEntry;

// Ce que le serveur HTTP aura le droit de consulter
pub struct AppStateServeur {
    // Si déverrouillé, contient la liste des entrées, sinon None
    pub coffre_ouvert: Option<Vec<PasswordEntry>>, 
}

// Le type qu'on va partager entre egui et Axum
pub type SafeState = Arc<Mutex<AppStateServeur>>;

// 1. Structure de la requête venant du navigateur
#[derive(Deserialize)]
pub struct CredentialQuery {
    pub domain: String,
}

// 2. Structure de la réponse renvoyée au navigateur
#[derive(Serialize)]
pub struct CredentialResponse {
    pub username: String,
    pub password: String,
}

// 3. Fonction qui démarre le serveur HTTP dans un thread dédié
pub fn start_server(state: SafeState) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            //Configurer la politique CORS : autorise ton extension navigateur à faire des requêtes vers localhost
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);

            let app = Router::new()
                .route("/api/credentials", get(handle_get_credentials))
                .with_state(Arc::clone(&state))
                .layer(cors);

            //Définir l'adresse (127.0.0.1:8765), bind le TcpListener et lancer axum::serve
            let addr = SocketAddr::from(([127, 0, 0, 1], 8765));
            println!("Serveur démarré sur http://{}", addr);

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    });
}

// 4. Le handler qui traite la requête / répond à l'extension
pub async fn handle_get_credentials(
    State(state): State<SafeState>,
    Query(query): Query<CredentialQuery>,
) -> Json<Option<CredentialResponse>> {
    println!("Requête reçue pour le domaine : {}", query.domain);
// 1. On verrouille le Mutex pour lire les données en toute sécurité
let lock = state.lock().unwrap();
    
// 2. On vérifie si le coffre est déverrouillé
if let Some(ref entries) = lock.coffre_ouvert {
    // 3. On cherche une entrée dont l'URL ou le titre contient le domaine recherché
    let entree_trouvee = entries.iter().find(|e| {
        // On vérifie si l'URL contient le domaine
        let match_url = match &e.url {
            Some(u) => u.to_lowercase().contains(&query.domain.to_lowercase()),
            None => false,
        };

        // On vérifie si le titre contient le domaine
        let match_title = e.title.to_lowercase().contains(&query.domain.to_lowercase());

        match_url || match_title
    });

    // 4. Si trouvée, on retourne les identifiants !
    if let Some(entree) = entree_trouvee {
        return Json(Some(CredentialResponse {
            username: entree.username.clone(),
            password: entree.password.clone(),
        }));
    }
}
// Si le coffre est verrouillé ou qu'aucune entrée ne correspond
Json(None)

}