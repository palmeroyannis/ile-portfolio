//! Les trois lieux de projet, détaillés : l'usine, le potager et le sommet de la colline.
//! Les socles et les objets provisoires des projets restent dans `monde.rs`.

use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::geo::{matiere_couleurs, nuance, Constructeur};
use crate::monde::{pave, sur_le_sol, COLLINE, PLACE, POTAGER, USINE};
use crate::rendu::{hasard, matiere_mate, texture_grain, texture_terre_cuite};

// --- Couleurs ---
const BETON: Color = Color::srgb(0.64, 0.62, 0.58);
const BRIQUE: Color = Color::srgb(0.72, 0.36, 0.22);
const TOIT: Color = Color::srgb(0.42, 0.47, 0.55);
const VITRE: Color = Color::srgb(0.20, 0.32, 0.46);
const CADRE: Color = Color::srgb(0.88, 0.88, 0.84);
const METAL: Color = Color::srgb(0.62, 0.65, 0.68);
const METAL_SOMBRE: Color = Color::srgb(0.32, 0.35, 0.39);
const BOIS: Color = Color::srgb(0.60, 0.43, 0.26);
const BOIS_SOMBRE: Color = Color::srgb(0.42, 0.30, 0.18);
const TERREAU: Color = Color::srgb(0.34, 0.23, 0.14);
const VERT: Color = Color::srgb(0.28, 0.62, 0.22);
const VERT_CLAIR: Color = Color::srgb(0.55, 0.80, 0.30);
const PIERRE: Color = Color::srgb(0.66, 0.64, 0.60);

pub struct LieuxPlugin;

impl Plugin for LieuxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_lieux)
            .add_systems(Update, balancer);
    }
}

/// Objet qui oscille doucement autour de la verticale (manche à air).
#[derive(Component)]
struct Balancement {
    base: Quat,
}

fn creer_lieux(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let grain = images.add(texture_grain());
    let simple = materials.add(matiere_couleurs(grain.clone(), false));
    let double = materials.add(matiere_couleurs(grain, true));
    let brique = materials.add(matiere_mate(
        images.add(texture_terre_cuite()),
        Vec2::new(8.0, 5.0),
    ));

    creer_usine(&mut commands, &mut meshes, &simple, &brique);
    creer_potager(&mut commands, &mut meshes, &simple, &double);
    creer_colline(&mut commands, &mut meshes, &simple, &double);
}

fn solide(commands: &mut Commands, centre: Vec3, taille: Vec3, rotation: Quat) {
    commands.spawn((
        Transform::from_translation(centre).with_rotation(rotation),
        RigidBody::Static,
        Collider::cuboid(taille.x, taille.y, taille.z),
    ));
}

// ---------------------------------------------------------------------------
// Usine
// ---------------------------------------------------------------------------

/// Fenêtre posée sur un mur : cadre, vitre, croisillons et appui.
fn fenetre(b: &mut Constructeur, centre: Vec3, normale: Vec3, largeur: f32, hauteur: f32) {
    let rotation = Quat::from_rotation_arc(Vec3::Z, normale);
    b.boite(
        centre + normale * 0.03,
        Vec3::new(largeur + 0.2, hauteur + 0.2, 0.1),
        rotation,
        CADRE,
    );
    b.boite(
        centre + normale * 0.05,
        Vec3::new(largeur, hauteur, 0.1),
        rotation,
        VITRE,
    );
    b.boite(
        centre + normale * 0.08,
        Vec3::new(0.06, hauteur, 0.1),
        rotation,
        CADRE,
    );
    b.boite(
        centre + normale * 0.08,
        Vec3::new(largeur, 0.06, 0.1),
        rotation,
        CADRE,
    );
    b.boite(
        centre - Vec3::Y * (hauteur / 2.0 + 0.12) + normale * 0.1,
        Vec3::new(largeur + 0.3, 0.08, 0.22),
        rotation,
        BETON,
    );
}

fn creer_usine(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    simple: &Handle<StandardMaterial>,
    brique: &Handle<StandardMaterial>,
) {
    let base = sur_le_sol(USINE, 0.0);
    let o = |x: f32, y: f32, z: f32| base + Vec3::new(x, y, z);
    let mut b = Constructeur::default();

    // Murs en briques (12 × 5 × 8 m), façade au sud (z = +2)
    pave(
        commands,
        meshes,
        brique,
        Vec3::new(12.0, 5.0, 8.0),
        o(0.0, 2.5, -2.0),
        Quat::IDENTITY,
        true,
    );
    // Soubassement en béton, qui descend sous le terrain
    b.boite(
        o(0.0, 0.15, -2.0),
        Vec3::new(12.3, 1.0, 8.3),
        Quat::IDENTITY,
        BETON,
    );
    // Corniche
    b.boite(
        o(0.0, 5.0, -2.0),
        Vec3::new(12.3, 0.18, 8.3),
        Quat::IDENTITY,
        nuance(BETON, 0.9),
    );

    // Fenêtres en façade et sur les côtés
    for x in [-4.9, 1.0, 3.6] {
        fenetre(&mut b, o(x, 3.0, 2.0), Vec3::Z, 1.4, 1.8);
    }
    for z in [-4.5, -2.0, 0.5] {
        fenetre(&mut b, o(6.0, 3.0, z), Vec3::X, 1.4, 1.8);
        fenetre(&mut b, o(-6.0, 3.0, z), Vec3::NEG_X, 1.4, 1.8);
    }
    for x in [-3.6, 0.0, 3.6] {
        fenetre(&mut b, o(x, 3.0, -6.0), Vec3::NEG_Z, 1.4, 1.8);
    }

    // Porte roulante à lames
    b.boite(
        o(-2.0, 1.65, 2.02),
        Vec3::new(2.8, 3.3, 0.14),
        Quat::IDENTITY,
        METAL_SOMBRE,
    );
    for k in 0..10 {
        let teinte = if k % 2 == 0 {
            METAL
        } else {
            nuance(METAL, 0.85)
        };
        b.boite(
            o(-2.0, 0.2 + 0.29 * k as f32, 2.09),
            Vec3::new(2.4, 0.26, 0.08),
            Quat::IDENTITY,
            teinte,
        );
    }
    // Plaque au-dessus de la porte (sans texte)
    b.boite(
        o(-2.0, 3.75, 2.06),
        Vec3::new(2.4, 0.5, 0.08),
        Quat::IDENTITY,
        Color::srgb(0.06, 0.12, 0.24),
    );
    b.boite(
        o(-2.0, 3.6, 2.11),
        Vec3::new(2.0, 0.05, 0.02),
        Quat::IDENTITY,
        Color::srgb(0.20, 0.82, 0.80),
    );

    // Toit en dents de scie : pans inclinés et verrières verticales tournées vers l'est
    let (avant, arriere) = (2.15, -6.15);
    let (bas, haut) = (5.0, 6.6);
    for i in 0..3 {
        let x0 = -6.0 + 4.0 * i as f32;
        let x1 = x0 + 4.0;
        let pan = nuance(TOIT, 0.95 + 0.08 * i as f32);
        b.quad(
            o(x0, bas, avant),
            o(x1, haut, avant),
            o(x1, haut, arriere),
            o(x0, bas, arriere),
            pan,
        );
        b.quad(
            o(x1, bas, avant),
            o(x1, bas, arriere),
            o(x1, haut, arriere),
            o(x1, haut, avant),
            VITRE,
        );
        let mut z = arriere + 1.0;
        while z < avant - 0.5 {
            b.boite(
                o(x1 + 0.03, (bas + haut) / 2.0, z),
                Vec3::new(0.06, haut - bas, 0.08),
                Quat::IDENTITY,
                CADRE,
            );
            z += 1.4;
        }
        b.triangle(
            o(x0, bas, avant),
            o(x1, bas, avant),
            o(x1, haut, avant),
            BRIQUE,
        );
        b.triangle(
            o(x0, bas, arriere),
            o(x1, haut, arriere),
            o(x1, bas, arriere),
            BRIQUE,
        );
        // Faîtage
        b.boite(
            o(x1 - 0.05, haut + 0.04, (avant + arriere) / 2.0),
            Vec3::new(0.2, 0.1, avant - arriere),
            Quat::IDENTITY,
            METAL_SOMBRE,
        );
    }

    // Cheminée avec un rebord
    pave(
        commands,
        meshes,
        brique,
        Vec3::new(1.2, 9.0, 1.2),
        o(4.5, 4.5, -4.5),
        Quat::IDENTITY,
        true,
    );
    b.boite(
        o(4.5, 9.0, -4.5),
        Vec3::new(1.5, 0.3, 1.5),
        Quat::IDENTITY,
        METAL_SOMBRE,
    );

    // Gouttière et descente
    b.cylindre(
        o(-6.1, 4.85, 2.2),
        o(6.1, 4.85, 2.2),
        0.09,
        0.09,
        6,
        METAL,
        true,
    );
    b.cylindre(
        o(5.8, 4.85, 2.2),
        o(5.8, 0.2, 2.2),
        0.07,
        0.07,
        6,
        METAL,
        true,
    );

    // Dalle en béton devant l'entrée
    b.boite(
        o(0.0, 0.0, 3.9),
        Vec3::new(9.0, 0.14, 3.5),
        Quat::IDENTITY,
        nuance(BETON, 1.05),
    );

    // Tonneaux le long du mur est
    for (k, (x, z)) in [(6.9, 1.0), (6.9, 0.1), (7.6, 0.55)]
        .into_iter()
        .enumerate()
    {
        let couleur = if k == 1 {
            Color::srgb(0.18, 0.32, 0.62)
        } else {
            Color::srgb(0.70, 0.18, 0.15)
        };
        b.cylindre(o(x, 0.0, z), o(x, 0.95, z), 0.36, 0.36, 8, couleur, true);
        for y in [0.2, 0.75] {
            b.cylindre(
                o(x, y, z),
                o(x, y + 0.05, z),
                0.38,
                0.38,
                8,
                METAL_SOMBRE,
                false,
            );
        }
        commands.spawn((
            Transform::from_translation(o(x, 0.47, z)),
            RigidBody::Static,
            Collider::cylinder(0.38, 0.95),
        ));
    }

    // Caisses empilées près de la façade
    for (x, y, z, angle) in [
        (-5.2, 0.4, 3.0, 0.1),
        (-4.3, 0.4, 3.2, -0.15),
        (-4.8, 1.2, 3.05, 0.3),
    ] {
        let rotation = Quat::from_rotation_y(angle);
        b.boite(
            o(x, y, z),
            Vec3::splat(0.8),
            rotation,
            nuance(BOIS, 0.9 + 0.2 * hasard((x * 10.0) as u32, 1, 70)),
        );
        // Renforts sombres sur les arêtes verticales
        for (dx, dz) in [(-0.38, 0.41), (0.38, 0.41), (-0.38, -0.41), (0.38, -0.41)] {
            b.boite(
                o(x, y, z) + rotation * Vec3::new(dx, 0.0, dz),
                Vec3::new(0.06, 0.8, 0.02),
                rotation,
                BOIS_SOMBRE,
            );
        }
        solide(commands, o(x, y, z), Vec3::splat(0.8), rotation);
    }

    commands.spawn((
        Mesh3d(meshes.add(b.construire())),
        MeshMaterial3d(simple.clone()),
    ));
}

// ---------------------------------------------------------------------------
// Potager
// ---------------------------------------------------------------------------

fn creer_potager(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    simple: &Handle<StandardMaterial>,
    double: &Handle<StandardMaterial>,
) {
    let c = sur_le_sol(POTAGER, 0.0);
    let o = |x: f32, y: f32, z: f32| c + Vec3::new(x, y, z);
    let mut b = Constructeur::default();
    let mut feuilles = Constructeur::default();

    // Quatre bacs surélevés en planches
    for (rang, z) in [-3.0_f32, -1.0, 1.0, 3.0].into_iter().enumerate() {
        let centre = o(2.5, 0.0, z);
        for dz in [-0.5, 0.5] {
            b.boite(
                centre + Vec3::new(0.0, 0.16, dz),
                Vec3::new(6.5, 0.32, 0.1),
                Quat::IDENTITY,
                BOIS,
            );
        }
        for dx in [-3.2, 3.2] {
            b.boite(
                centre + Vec3::new(dx, 0.16, 0.0),
                Vec3::new(0.1, 0.32, 1.1),
                Quat::IDENTITY,
                BOIS_SOMBRE,
            );
        }
        b.boite(
            centre + Vec3::new(0.0, 0.14, 0.0),
            Vec3::new(6.3, 0.26, 0.9),
            Quat::IDENTITY,
            TERREAU,
        );
        solide(
            commands,
            centre + Vec3::Y * 0.16,
            Vec3::new(6.5, 0.32, 1.1),
            Quat::IDENTITY,
        );

        let sol = 0.28;
        match rang {
            // Salades
            0 => {
                for k in 0..6 {
                    let p = centre + Vec3::new(-2.5 + k as f32, sol, 0.0);
                    feuilles.losange(p + Vec3::Y * 0.1, Vec3::new(0.26, 0.13, 0.26), VERT_CLAIR);
                    feuilles.losange(
                        p + Vec3::Y * 0.18,
                        Vec3::new(0.15, 0.14, 0.15),
                        nuance(VERT_CLAIR, 1.15),
                    );
                }
            }
            // Tomates sur tuteurs
            1 => {
                for (k, x) in [-2.2_f32, -0.7, 0.8, 2.3].into_iter().enumerate() {
                    let p = centre + Vec3::new(x, sol, 0.0);
                    b.boite(
                        p + Vec3::new(0.12, 0.55, 0.0),
                        Vec3::new(0.04, 1.1, 0.04),
                        Quat::IDENTITY,
                        BOIS,
                    );
                    feuilles.losange(p + Vec3::Y * 0.35, Vec3::new(0.26, 0.3, 0.26), VERT);
                    feuilles.losange(
                        p + Vec3::Y * 0.75,
                        Vec3::new(0.22, 0.28, 0.22),
                        nuance(VERT, 1.1),
                    );
                    for t in 0..3 {
                        let a = t as f32 * 2.1 + k as f32;
                        feuilles.losange(
                            p + Vec3::new(a.cos() * 0.24, 0.35 + 0.18 * t as f32, a.sin() * 0.24),
                            Vec3::splat(0.07),
                            Color::srgb(0.88, 0.16, 0.10),
                        );
                    }
                }
            }
            // Carottes : fanes en éventail
            2 => {
                for k in 0..10 {
                    let p = centre + Vec3::new(-2.7 + 0.6 * k as f32, sol, 0.0);
                    feuilles.losange(
                        p,
                        Vec3::new(0.05, 0.03, 0.05),
                        Color::srgb(0.95, 0.50, 0.12),
                    );
                    for f in 0..3 {
                        let a = f as f32 * TAU / 3.0 + k as f32;
                        let d = Vec3::new(a.cos(), 0.0, a.sin());
                        let travers = Vec3::new(-d.z, 0.0, d.x) * 0.04;
                        feuilles.triangle_double(
                            p + travers,
                            p - travers,
                            p + Vec3::Y * 0.32 + d * 0.12,
                            nuance(VERT_CLAIR, 0.9),
                        );
                    }
                }
            }
            // Tournesols, têtes tournées vers l'ouest (vers la place)
            _ => {
                for k in 0..4 {
                    let p = centre + Vec3::new(-2.4 + 1.6 * k as f32, sol, 0.0);
                    let haut = 1.4 + 0.2 * hasard(k, 3, 71);
                    b.cylindre(p, p + Vec3::Y * haut, 0.035, 0.03, 5, VERT, false);
                    for (dy, dx) in [(0.5, 0.25), (0.85, -0.25)] {
                        feuilles.triangle_double(
                            p + Vec3::Y * dy,
                            p + Vec3::new(dx, dy + 0.05, 0.12),
                            p + Vec3::new(dx * 1.2, dy + 0.12, -0.1),
                            VERT,
                        );
                    }
                    let tete = p + Vec3::Y * haut;
                    feuilles.losange(
                        tete,
                        Vec3::new(0.05, 0.30, 0.30),
                        Color::srgb(1.0, 0.80, 0.10),
                    );
                    feuilles.losange(
                        tete - Vec3::X * 0.04,
                        Vec3::new(0.04, 0.13, 0.13),
                        Color::srgb(0.38, 0.24, 0.10),
                    );
                }
            }
        }
    }

    // Clôture à poteaux et deux lisses, sur trois côtés (l'entrée est à l'ouest)
    let cotes = [
        (Vec3::new(-2.0, 0.0, -4.4), Vec3::new(7.0, 0.0, -4.4)),
        (Vec3::new(-2.0, 0.0, 4.4), Vec3::new(7.0, 0.0, 4.4)),
        (Vec3::new(7.0, 0.0, -4.4), Vec3::new(7.0, 0.0, 4.4)),
    ];
    for (debut, fin) in cotes {
        let longueur = debut.distance(fin);
        let direction = (fin - debut) / longueur;
        let rotation = Quat::from_rotation_arc(Vec3::X, direction);
        let poteaux = (longueur / 1.5).round() as i32;
        for k in 0..=poteaux {
            let p = c + debut + direction * (longueur * k as f32 / poteaux as f32);
            b.boite(
                p + Vec3::Y * 0.35,
                Vec3::new(0.12, 0.9, 0.12),
                Quat::IDENTITY,
                BOIS_SOMBRE,
            );
            b.boite(
                p + Vec3::Y * 0.83,
                Vec3::new(0.16, 0.06, 0.16),
                Quat::IDENTITY,
                BOIS,
            );
        }
        let milieu = c + (debut + fin) / 2.0;
        for y in [0.32, 0.62] {
            b.boite(
                milieu + Vec3::Y * y,
                Vec3::new(longueur, 0.08, 0.05),
                rotation,
                BOIS,
            );
        }
        solide(
            commands,
            milieu + Vec3::Y * 0.45,
            Vec3::new(longueur, 0.9, 0.15),
            rotation,
        );
    }

    // Arrosoir près de l'entrée
    let arrosoir = o(-2.2, 0.0, 2.6);
    b.boite(
        arrosoir + Vec3::Y * 0.16,
        Vec3::new(0.34, 0.32, 0.2),
        Quat::IDENTITY,
        Color::srgb(0.20, 0.50, 0.36),
    );
    b.cylindre(
        arrosoir + Vec3::new(0.15, 0.1, 0.0),
        arrosoir + Vec3::new(0.42, 0.36, 0.0),
        0.035,
        0.03,
        5,
        Color::srgb(0.20, 0.50, 0.36),
        true,
    );
    b.boite(
        arrosoir + Vec3::Y * 0.37,
        Vec3::new(0.24, 0.04, 0.04),
        Quat::IDENTITY,
        Color::srgb(0.16, 0.40, 0.30),
    );

    // Pas japonais du chemin jusqu'au socle
    for k in 0..4 {
        let p = o(-0.4 - 0.9 * k as f32, 0.02, 0.15 * (k % 2) as f32 - 0.1);
        b.boite(
            p,
            Vec3::new(0.6, 0.08, 0.5),
            Quat::from_rotation_y(0.3 * k as f32),
            PIERRE,
        );
    }

    commands.spawn((
        Mesh3d(meshes.add(b.construire())),
        MeshMaterial3d(simple.clone()),
    ));
    commands.spawn((
        Mesh3d(meshes.add(feuilles.construire())),
        MeshMaterial3d(double.clone()),
    ));
}

// ---------------------------------------------------------------------------
// Sommet de la colline
// ---------------------------------------------------------------------------

fn creer_colline(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    simple: &Handle<StandardMaterial>,
    double: &Handle<StandardMaterial>,
) {
    let sommet = sur_le_sol(COLLINE, 0.0);
    let mut b = Constructeur::default();

    // Dallage octogonal autour du socle
    let cotes = 8;
    let rayon = 2.4;
    for k in 0..cotes {
        let a0 = k as f32 / cotes as f32 * TAU;
        let a1 = (k + 1) as f32 / cotes as f32 * TAU;
        let p0 = Vec3::new(a0.cos(), 0.0, a0.sin()) * rayon;
        let p1 = Vec3::new(a1.cos(), 0.0, a1.sin()) * rayon;
        let dessus = Vec3::Y * 0.08;
        let teinte = nuance(PIERRE, 0.9 + 0.2 * hasard(k, 0, 72));
        b.triangle(
            sommet + dessus,
            sommet + p1 + dessus,
            sommet + p0 + dessus,
            teinte,
        );
        b.quad(
            sommet + p0 - Vec3::Y * 0.3,
            sommet + p0 + dessus,
            sommet + p1 + dessus,
            sommet + p1 - Vec3::Y * 0.3,
            nuance(PIERRE, 0.8),
        );
    }

    // Banc tourné vers la mer (vers l'extérieur de l'île)
    let dehors = COLLINE.normalize();
    let vers_mer = Vec3::new(dehors.x, 0.0, dehors.y);
    let banc = sommet + vers_mer * 2.9;
    let rotation = Quat::from_rotation_arc(Vec3::Z, vers_mer);
    let r = |v: Vec3| banc + rotation * v;
    b.boite(
        r(Vec3::new(0.0, 0.45, 0.0)),
        Vec3::new(1.4, 0.08, 0.42),
        rotation,
        BOIS,
    );
    b.boite(
        r(Vec3::new(0.0, 0.78, -0.2)),
        Vec3::new(1.4, 0.3, 0.06),
        rotation,
        BOIS,
    );
    for (dx, dz) in [(-0.6, 0.15), (0.6, 0.15), (-0.6, -0.17), (0.6, -0.17)] {
        b.boite(
            r(Vec3::new(dx, 0.2, dz)),
            Vec3::new(0.08, 0.5, 0.08),
            rotation,
            BOIS_SOMBRE,
        );
    }
    solide(
        commands,
        r(Vec3::new(0.0, 0.3, 0.0)),
        Vec3::new(1.4, 0.6, 0.45),
        rotation,
    );

    // Panneau indicateur à l'arrivée du chemin, flèches vers le potager et l'usine
    let vers_place = (PLACE - COLLINE).normalize();
    let pied_panneau = sur_le_sol(
        COLLINE + vers_place * 3.3 + Vec2::new(-vers_place.y, vers_place.x) * 1.3,
        0.0,
    );
    b.cylindre(
        pied_panneau - Vec3::Y * 0.2,
        pied_panneau + Vec3::Y * 1.9,
        0.06,
        0.06,
        6,
        BOIS_SOMBRE,
        true,
    );
    for (cible, y) in [(POTAGER, 1.65), (USINE, 1.35)] {
        let d = (cible - COLLINE).normalize();
        let d = Vec3::new(d.x, 0.0, d.y);
        let rot = Quat::from_rotation_arc(Vec3::X, d);
        let centre = pied_panneau + Vec3::Y * y + d * 0.45;
        b.boite(centre, Vec3::new(0.8, 0.2, 0.05), rot, BOIS);
        let pointe = centre + d * 0.4;
        let cote = rot * Vec3::Y * 0.14;
        b.triangle(pointe + cote, pointe - cote, pointe + d * 0.18, BOIS);
        b.triangle(pointe - cote, pointe + cote, pointe + d * 0.18, BOIS);
    }

    // Mât de la manche à air
    let mat = sommet + Vec3::new(2.0, 0.0, -2.2);
    b.cylindre(
        mat - Vec3::Y * 0.2,
        mat + Vec3::Y * 3.4,
        0.05,
        0.04,
        6,
        Color::srgb(0.85, 0.85, 0.85),
        true,
    );

    commands.spawn((
        Mesh3d(meshes.add(b.construire())),
        MeshMaterial3d(simple.clone()),
    ));

    // Manche à air rayée orange et blanc, qui se balance au vent
    let mut manche = Constructeur::default();
    for k in 0..5 {
        let x0 = k as f32 * 0.3;
        let x1 = x0 + 0.3;
        let couleur = if k % 2 == 0 {
            Color::srgb(0.95, 0.45, 0.10)
        } else {
            Color::srgb(0.96, 0.96, 0.94)
        };
        manche.cylindre(
            Vec3::new(x0, 0.0, 0.0),
            Vec3::new(x1, 0.0, 0.0),
            0.22 - 0.025 * k as f32,
            0.22 - 0.025 * (k + 1) as f32,
            7,
            couleur,
            false,
        );
    }
    let base = Quat::from_rotation_y(-0.6) * Quat::from_rotation_z(-0.15);
    commands.spawn((
        Balancement { base },
        Mesh3d(meshes.add(manche.construire())),
        MeshMaterial3d(double.clone()),
        Transform::from_translation(mat + Vec3::Y * 3.3).with_rotation(base),
    ));
}

fn balancer(time: Res<Time>, mut objets: Query<(&mut Transform, &Balancement)>) {
    let t = time.elapsed_secs();
    for (mut transform, balancement) in &mut objets {
        let lacet = 0.35 * (t * 0.7).sin() + 0.1 * (t * 2.3).sin();
        let tangage = 0.08 * (t * 1.9).sin();
        transform.rotation =
            balancement.base * Quat::from_rotation_y(lacet) * Quat::from_rotation_z(tangage);
    }
}
