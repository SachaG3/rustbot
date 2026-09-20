# SoyerBOT - Version Rust

Une implémentation optimisée en Rust du bot SoyerBOT utilisant Serenity.

## Fonctionnalités

- **Commandes de base** : afficher des informations sur le serveur, lien vers le serveur de jeux, et lier son compte Discord au site web
- **Système de profil** : créer un profil, afficher son score
- **Jeux** : deviner un nombre entre 1 et 1000, deviner un nombre entre 1 et 10, jouer au Blackjack
- **Spam** : répéter un message plusieurs fois, avec ou sans TTS

## Prérequis

- Rust 1.65.0 ou plus récent
- Une base de données MySQL
- Un token de bot Discord

## Configuration

1. Créez un fichier `.env` à la racine du projet avec les variables suivantes :

```
TOKEN_DISCORD=votre_token_discord
DB_HOST=localhost
DB_USER=votre_utilisateur
DB_PASSWORD=votre_mot_de_passe
DB_DATABASE=votre_base_de_donnees
DB_PORT=3306
OWNER_ID=votre_id_discord
```

2. Assurez-vous que votre base de données MySQL est configurée avec les tables nécessaires.

## Installation et Exécution

1. Clonez ce dépôt :
```
git clone https://github.com/votre-nom/soyer-bot.git
cd soyer-bot
```

2. Construisez le projet :
```
cargo build --release
```

3. Lancez le bot :
```
cargo run --release
```

## Optimisations par rapport à la version Python

1. **Performance** : Rust est un langage compilé qui offre des performances proches du C/C++, ce qui rend le bot plus rapide et plus économe en ressources.

2. **Sécurité de la mémoire** : Rust garantit la sécurité de la mémoire à la compilation, ce qui élimine les erreurs courantes comme les dépassements de tampon et les utilisations après libération.

3. **Concurrence et Parallelisme** : Le modèle de concurrence de Rust basé sur les futurs et les tâches asynchrones est plus efficace pour gérer de nombreuses connexions simultanées.

4. **Gestion des erreurs** : Le système de gestion d'erreurs de Rust oblige à traiter explicitement les erreurs potentielles, rendant le code plus robuste.

5. **Optimisation de la base de données** : Utilisation de connexions poolées pour optimiser les interactions avec la base de données.

6. **Limitation des abus** : Ajout de limites pour les commandes de spam pour éviter les abus.

## Commandes

Toutes les commandes commencent par le préfixe `^^` :

- `^^serverinfo` - Affiche des informations sur le serveur
- `^^jeux` - Affiche un lien vers le serveur de jeux
- `^^link` - Génère un lien pour connecter votre compte Discord au site web
- `^^np` - Crée un nouveau profil
- `^^score` - Affiche votre score actuel
- `^^juste` - Jeu pour deviner un nombre entre 1 et 1000
- `^^usd` - Jeu pour deviner un nombre entre 1 et 10
- `^^bj` - Joue au Blackjack contre le bot
- `^^rp` - Répète un message plusieurs fois
- `^^rpt` - Répète un message plusieurs fois avec TTS

### Vie de la maison et communauté

- `^^missions` - Affiche les missions hebdomadaires sans série punitive
- `^^expedition <id> <courte|moyenne|longue>` - Envoie un chat en promenade sans danger
- `^^expeditions` - Affiche les promenades en cours
- `^^retour <id>` - Accueille un chat revenu sain et sauf
- `^^decorations` - Affiche les décorations débloquées
- `^^decorer <clé>` / `^^ranger <clé>` - Personnalise la maison
- `^^defi` - Affiche le défi communautaire en cours
- `^^participer_defi <id>` - Participe avec un chat, sans risque pour lui
- `^^hebdo` - Affiche le résumé de la semaine

Les défis apparaissent aléatoirement dans `CAT_CHANNEL_ID`. Ils comprennent le chien turbulent, mais aussi des activités coopératives comme le carton géant, l'orage solidaire, la pelote géante et le pique-nique. Une issue insuffisante met toujours les chats à l'abri : aucun chat ne peut être blessé, perdu ou retiré à son propriétaire.

Le dimanche après 20 h (heure de Paris), le bot publie une fois le classement des personnes les plus régulières sur `^^cat`, puis tous les chats de collection apparus pendant la semaine.

## Contribution

Les contributions sont les bienvenues ! N'hésitez pas à soumettre des issues ou des pull requests.

## Licence

Ce projet est sous licence MIT. 