//! Le contenu des fiches projet. Pour modifier un texte ou ajouter une photo, c'est ici.
//! Les photos sont dans le dossier `assets/projets`.

pub struct Projet {
    pub titre: &'static str,
    pub sous_titre: &'static str,
    pub paragraphes: &'static [&'static str],
    /// Paires (libellé, valeur) affichées sous « Chiffres clés »
    pub chiffres: &'static [(&'static str, &'static str)],
    pub photos: &'static [&'static str],
}

/// L'ordre correspond aux lieux de l'île : colline, potager, usine.
pub const PROJETS: [Projet; 3] = [
    Projet {
        titre: "Randolienne",
        sous_titre: "Projet d'école · 2024 · Éolienne portable",
        paragraphes: &[
            "Éolienne Savonius compacte qui recharge un téléphone en randonnée, uniquement avec le vent. Elle se monte sans outil sur un bâton de randonnée, qui sert d'axe principal.",
            "Trois pales en toile de parapente, un haubanage par trois câbles et un train d'engrenages imprimé en 3D qui amène le générateur à 1500 tr/min. Le prototype a été validé en soufflerie.",
        ],
        chiffres: &[
            ("Masse", "1,19 kg"),
            ("Démarrage", "3 m/s de vent"),
            ("Recharge", "5 V stables à 6 m/s"),
            ("Tenue au vent", "12 m/s"),
            ("Multiplicateur", "9,375:1"),
            ("Montage", "8 min, sans outil"),
        ],
        photos: &[
            "projets/randolienne-cad-1.jpg",
            "projets/randolienne-cad-2.jpg",
            "projets/randolienne-cad-3.jpg",
        ],
    },
    Projet {
        titre: "Smart plant pot",
        sous_titre: "Projet personnel · 2026, en cours · Pot connecté",
        paragraphes: &[
            "Pot connecté qui surveille la santé d'une plante. Un capteur capacitif mesure l'humidité du sol ; il est intégré dans un boîtier nervuré imprimé en 3D, qui garde l'électronique discrète.",
            "Pour l'instant, seule la partie matérielle existe : le capteur fonctionne et le câblage est prêt pour un microcontrôleur. Prochaine étape : enregistrer les mesures et envoyer des alertes d'arrosage.",
        ],
        chiffres: &[
            ("Capteur", "Humidité du sol, capacitif"),
            ("Boîtier", "Nervuré, imprimé en 3D"),
            ("Électronique", "Câblage prêt pour microcontrôleur"),
        ],
        photos: &["projets/plantpot.jpg"],
    },
    Projet {
        titre: "Imprimante 3D à chauffage local",
        sous_titre: "Projet d'école · 2023 · Preuve de concept",
        paragraphes: &[
            "Un laser préchauffe la couche inférieure juste avant le dépôt du filament, pour mieux souder les couches entre elles et limiter le gauchissement, surtout avec des matériaux techniques comme l'ABS.",
            "Le laser reste fixé au châssis d'une Prusa i3 ; une fibre optique amène le faisceau jusqu'à une couronne motorisée qui tourne autour de la buse. Un script Python retraite le G-code du slicer pour synchroniser la rotation avec la trajectoire.",
        ],
        chiffres: &[
            ("Rotation", "360° autour de la buse"),
            ("Point chauffé", "5 à 10 mm devant la buse"),
            ("Résolution angulaire", "0,4°"),
            ("Base", "Prusa i3 modifiée"),
            ("Pilotage", "Python, G-code, Pronterface"),
        ],
        photos: &[
            "projets/heating.jpg",
            "projets/heating-hardware.jpg",
            "projets/heating-detail.jpg",
        ],
    },
];
