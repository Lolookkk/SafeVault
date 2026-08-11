async function autofill() {
    // 1. On extrait le nom de domaine de la page actuelle (ex: "github")
    const hostname = window.location.hostname; // ex: "github.com"
    const domain = hostname.replace('www.', '').split('.')[0]; // extrait "github"

    // 2. On demande au background.js de contacter le serveur Rust
    chrome.runtime.sendMessage(
        { action: "fetch_credentials", domain: domain },
        (response) => {
            // Si le serveur a renvoyé un identifiant et un mot de passe
            if (response && response.username && response.password) {
                
                // 🧩 TROU 3 : Trouve le champ du NOM D'UTILISATEUR ou EMAIL dans la page HTML
                // Astuce : utilise document.querySelector avec les sélecteurs CSS classiques d'un input de login
                // (ex: input[type="text"], input[type="email"], input[name="login"])
                const userInput = document.querySelector('input[type="text"], input[type="email"], input[name="login"]');

                // 🧩 TROU 4 : Trouve le champ du MOT DE PASSE dans la page HTML
                // Astuce : les champs mot de passe ont presque toujours le type "password"
                const passInput = document.querySelector('input[type="password"]');

                // Injection des valeurs dans les champs s'ils existent
                if (userInput) {
                    userInput.value = response.username;
                    // Déclenche les événements JS du site pour que le formulaire valide la saisie
                    userInput.dispatchEvent(new Event('input', { bubbles: true }));
                }

                if (passInput) {
                    // 🧩 TROU 5 : Assigne le mot de passe reçu (response.password) à la valeur du champ passInput
                    passInput.value = response.password;
                    
                    passInput.dispatchEvent(new Event('input', { bubbles: true }));
                }

                console.log("✅ SafeVault : Identifiants remplis automatiquement !");
            }
        }
    );
}

// Exécute le remplissage automatique 1 seconde après le chargement de la page
setTimeout(autofill, 1000);