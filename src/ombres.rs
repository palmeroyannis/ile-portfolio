//! Ombres et occlusion « peintes » dans les couleurs du sol, calculées une fois au démarrage,
//! comme on le faisait sur GameCube : aucun coût à l'affichage.
//!
//! Chaque objet qui fait de l'ombre est décrit très simplement (un disque en hauteur pour la
//! couronne d'un palmier, un pavé pour l'usine). Pour savoir si un point du sol est à l'ombre,
//! on remonte vers le soleil et on regarde si on traverse un de ces volumes.

use bevy::prelude::*;

use crate::decor::{palmiers, rochers};
use crate::monde::{sur_le_sol, USINE};
use crate::rendu::direction_soleil;

// --- Réglages ---
/// Teinte d'une zone en pleine ombre (bleutée : l'ombre reçoit la lumière du ciel)
pub const TEINTE_OMBRE: Vec3 = Vec3::new(0.60, 0.63, 0.74);
const FORCE_OMBRE_PALMES: f32 = 0.75; // les palmes laissent passer un peu de lumière

pub enum Occultant {
    /// Disque horizontal à une altitude donnée (couronne de palmier, rocher, tronc)
    Disque {
        centre: Vec3,
        rayon: f32,
        force: f32,
    },
    /// Pavé vertical posé au sol (bâtiment, cheminée)
    Pave { min: Vec2, max: Vec2, haut: f32 },
}

/// La liste des objets qui font de l'ombre sur l'île.
pub fn occultants() -> Vec<Occultant> {
    let mut liste = Vec::new();

    for palmier in palmiers() {
        let sommet = palmier.sommet();
        liste.push(Occultant::Disque {
            centre: sommet - Vec3::Y * 0.5,
            rayon: 2.7,
            force: FORCE_OMBRE_PALMES,
        });
        // Le tronc : une série de petits disques le long de la courbe
        for k in 1..8 {
            let t = k as f32 / 8.0;
            let p = palmier.pied + Vec3::Y * palmier.hauteur_tronc * t + palmier.penche * t * t;
            liste.push(Occultant::Disque {
                centre: p,
                rayon: 0.24,
                force: 0.9,
            });
        }
    }

    for (centre, taille, _) in rochers() {
        liste.push(Occultant::Disque {
            centre: centre + Vec3::Y * taille.y * 0.3,
            rayon: taille.x.max(taille.z) * 0.9,
            force: 0.9,
        });
    }

    // Usine : murs et toit (en moyenne 6 m), puis la cheminée
    let base = sur_le_sol(USINE, 0.0);
    liste.push(Occultant::Pave {
        min: base.xz() + Vec2::new(-6.0, -6.0),
        max: base.xz() + Vec2::new(6.0, 2.0),
        haut: base.y + 6.0,
    });
    liste.push(Occultant::Pave {
        min: base.xz() + Vec2::new(3.9, -5.1),
        max: base.xz() + Vec2::new(5.1, -3.9),
        haut: base.y + 9.2,
    });

    liste
}

fn lissage(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn distance_rectangle(p: Vec2, min: Vec2, max: Vec2) -> f32 {
    let d = (min - p).max(p - max).max(Vec2::ZERO);
    d.length()
}

/// Proportion d'ombre (0 = plein soleil, 1 = pleine ombre) en un point du sol.
pub fn ombre(point: Vec3, occultants: &[Occultant]) -> f32 {
    let soleil = direction_soleil();
    // Déplacement horizontal quand on monte d'un mètre vers le soleil
    let pente = soleil.xz() / soleil.y;
    let mut resultat: f32 = 0.0;

    for occultant in occultants {
        match occultant {
            Occultant::Disque {
                centre,
                rayon,
                force,
            } => {
                let montee = centre.y - point.y;
                if montee <= 0.0 {
                    continue;
                }
                let q = point.xz() + pente * montee;
                let d = q.distance(centre.xz());
                let o = (1.0 - lissage(rayon * 0.75, rayon * 1.1, d)) * force;
                resultat = resultat.max(o);
            }
            Occultant::Pave { min, max, haut } => {
                let montee = haut - point.y;
                if montee <= 0.0 {
                    continue;
                }
                // On remonte le rayon vers le soleil par pas de 40 cm
                let pas = (montee / 0.4).ceil() as i32;
                let mut proche = f32::MAX;
                for k in 0..=pas {
                    let q = point.xz() + pente * (montee * k as f32 / pas as f32);
                    proche = proche.min(distance_rectangle(q, *min, *max));
                }
                resultat = resultat.max(1.0 - lissage(0.0, 0.35, proche));
            }
        }
        if resultat >= 1.0 {
            break;
        }
    }
    resultat
}

/// Occlusion ambiante au sol (0 = dégagé, 1 = très encaissé) : pied des murs, base des rochers
/// et des troncs.
pub fn occlusion(point: Vec3, occultants: &[Occultant]) -> f32 {
    let mut resultat: f32 = 0.0;
    for occultant in occultants {
        let o = match occultant {
            Occultant::Pave { min, max, haut } if *haut > point.y => {
                let d = distance_rectangle(point.xz(), *min, *max);
                0.55 * (1.0 - lissage(0.0, 1.8, d))
            }
            Occultant::Disque { centre, rayon, .. } if (centre.y - point.y).abs() < 1.6 => {
                let d = point.xz().distance(centre.xz()) - rayon * 0.8;
                0.45 * (1.0 - lissage(0.0, 0.9, d))
            }
            _ => 0.0,
        };
        resultat = resultat.max(o);
    }
    resultat
}

/// Applique ombre et occlusion à une couleur (valeurs linéaires).
pub fn assombrir(couleur: Vec3, ombre: f32, occlusion: f32) -> Vec3 {
    couleur * Vec3::ONE.lerp(TEINTE_OMBRE, ombre) * (1.0 - 0.5 * occlusion)
}

/// Même chose pour une couleur d'objet (herbe, fleurs).
pub fn ombrer_couleur(couleur: Color, ombre: f32, occlusion: f32) -> Color {
    let l = couleur.to_linear();
    let v = assombrir(Vec3::new(l.red, l.green, l.blue), ombre, occlusion);
    Color::linear_rgba(v.x, v.y, v.z, l.alpha)
}
