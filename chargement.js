// Écran de chargement : suit le téléchargement du jeu (trunk appelle ces fonctions).
// Deux phases : le téléchargement (barre qui avance), puis la préparation de l'île
// (balayage animé, qui continue même quand le navigateur est occupé à démarrer le jeu).
// L'écran disparaît ensuite tout seul, depuis le jeu, une fois l'île affichée.

export default function chargement() {
  const ecran = () => document.getElementById("chargement");
  const barre = () => document.getElementById("barre");
  const etat = () => document.getElementById("etat");
  const mo = (octets) => (octets / 1048576).toFixed(0);

  const preparation = () => {
    barre().style.width = "100%";
    ecran().classList.add("preparation");
    etat().textContent = "Préparation de l'île…";
  };

  return {
    onStart: () => {
      etat().textContent = "Téléchargement de l'île…";
    },
    onProgress: ({ current, total }) => {
      if (!total) {
        etat().textContent = `Téléchargement de l'île… ${mo(current)} Mo`;
        return;
      }
      if (current >= total) {
        preparation();
        return;
      }
      const part = current / total;
      barre().style.width = `${(part * 100).toFixed(1)}%`;
      etat().textContent = `Téléchargement de l'île… ${mo(current)} / ${mo(total)} Mo`;
    },
    onComplete: preparation,
    onSuccess: () => {},
    onFailure: (erreur) => {
      console.error(erreur);
      ecran().classList.remove("preparation");
      ecran().classList.add("echec");
      etat().textContent = "Le jeu n'a pas pu se lancer sur cet appareil.";
    },
  };
}
