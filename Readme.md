[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/emMZvU8G)
# SdE2 Devoir 3 Starter - Rusty Loader

## Solution

TODO Décrivez ici comment avez-vous résolu les devoirs. 
J'ai ajouté du code pour lire et analyser les segments du fichier ELF. Les segments sont ensuite affichés au format requis. J'ai également ajouté la gestion des adresses de base et du point d'entrée pour l'exécutable.
et J'ai implemente un gestionnaire de signal pour intercepter les défauts de page (SIGSEGV). Ce gestionnaire vérifiera si l'adresse fautive appartient à un segment du fichier ELF et chargera la page correspondante en mémoire si nécessaire.