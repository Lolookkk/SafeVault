// On écoute les requêtes venant de content.js
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.action === "fetch_credentials") {
        const domain = request.domain;
        
        // 🧩 TROU 1 : Construis l'URL complète de ton API Axum avec le paramètre domain
        // Exemple de résultat attendu : "http://127.0.0.1:8765/api/credentials?domain=" + domain
        const apiUrl = "http://127.0.0.1:8765/api/credentials?domain="+domain;

        // Appel HTTP vers ton serveur Rust
        fetch(apiUrl)
            .then(response => response.json())
            .then(data => {
                // 🧩 TROU 2 : Renvoie la réponse 'data' reçue du serveur HTTP à content.js
                // Astuce : utilise la fonction sendResponse(...)
                sendResponse(data);
            })
            .catch(error => {
                console.error("Erreur de connexion au coffre SafeVault:", error);
                sendResponse(null);
            });

        // Nécessaire pour indiquer que la réponse est asynchrone en JS
        return true; 
    }
});