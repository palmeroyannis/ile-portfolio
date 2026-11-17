//! Apparence du personnage : un ingénieur low-poly à facettes (t-shirt, pantalon de travail,
//! sac à dos), animé par le code. Sera remplacé par un modèle Blender si besoin.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::joueur::{Apparence, Joueur};

// --- Couleurs ---
const PEAU: Color = Color::srgb(0.87, 0.67, 0.52);
const CHEVEUX: Color = Color::srgb(0.20, 0.13, 0.08);
const T_SHIRT: Color = Color::srgb(0.20, 0.42, 0.62);
const PANTALON: Color = Color::srgb(0.30, 0.31, 0.27);
const CHAUSSURES: Color = Color::srgb(0.22, 0.16, 0.12);
const SAC: Color = Color::srgb(0.75, 0.38, 0.16);

// --- Animation ---
const FREQUENCE_PAS: f32 = 2.1; // plus grand = pas plus rapides pour une même vitesse
const AMPLITUDE_JAMBES: f32 = 0.75; // radians à pleine vitesse
const AMPLITUDE_BRAS: f32 = 0.65;
const REBOND: f32 = 0.05; // mètres
const DOUCEUR: f32 = 14.0;

// Le centre de la capsule physique est à mi-hauteur : les pieds sont à -0,85 m.
const HANCHES: f32 = -0.05;
const EPAULES: f32 = 0.47;

pub struct PersonnagePlugin;

impl Plugin for PersonnagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animer);
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
enum Membre {
    JambeGauche,
    JambeDroite,
    BrasGauche,
    BrasDroit,
}

/// Phase du cycle de marche, propre au personnage.
#[derive(Component, Default)]
pub struct Allure {
    phase: f32,
}

/// Maillage aux faces plates : chaque facette a sa propre normale, les arêtes se voient.
fn facettes(maillage: impl Into<Mesh>) -> Mesh {
    maillage.into().with_duplicated_vertices().with_computed_flat_normals()
}

pub fn construire(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut mat = |couleur: Color| {
        materials.add(StandardMaterial {
            base_color: couleur,
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        })
    };
    let peau = mat(PEAU);
    let cheveux = mat(CHEVEUX);
    let t_shirt = mat(T_SHIRT);
    let pantalon = mat(PANTALON);
    let chaussures = mat(CHAUSSURES);
    let sac = mat(SAC);

    let jambe = meshes.add(facettes(Cylinder::new(0.085, 0.80).mesh().resolution(6)));
    let bras = meshes.add(facettes(Cylinder::new(0.06, 0.52).mesh().resolution(6)));
    let main = meshes.add(facettes(Sphere::new(0.06).mesh().ico(0).unwrap()));
    let chaussure = meshes.add(Cuboid::new(0.14, 0.09, 0.26));
    let tronc = meshes.add(facettes(Cylinder::new(0.21, 0.56).mesh().resolution(6)));
    let tete = meshes.add(facettes(Sphere::new(1.0).mesh().ico(1).unwrap()));
    let cou = meshes.add(facettes(Cylinder::new(0.06, 0.12).mesh().resolution(5)));
    let boite = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    parent
        .spawn((Apparence, Allure::default(), Transform::default(), Visibility::default()))
        .with_children(|corps| {
            // Tronc (légèrement aplati d'avant en arrière)
            corps.spawn((
                Mesh3d(tronc),
                MeshMaterial3d(t_shirt.clone()),
                Transform::from_xyz(0.0, 0.24, 0.0).with_scale(Vec3::new(1.0, 1.0, 0.62)),
            ));
            // Ceinture / bassin
            corps.spawn((
                Mesh3d(boite.clone()),
                MeshMaterial3d(pantalon.clone()),
                Transform::from_xyz(0.0, -0.02, 0.0).with_scale(Vec3::new(0.38, 0.14, 0.22)),
            ));
            // Sac à dos (le personnage regarde vers -Z, le dos est donc en +Z)
            corps.spawn((
                Mesh3d(boite.clone()),
                MeshMaterial3d(sac.clone()),
                Transform::from_xyz(0.0, 0.28, 0.19).with_scale(Vec3::new(0.30, 0.38, 0.13)),
            ));
            corps.spawn((
                Mesh3d(boite.clone()),
                MeshMaterial3d(sac),
                Transform::from_xyz(0.0, 0.40, 0.26).with_scale(Vec3::new(0.24, 0.10, 0.05)),
            ));
            // Cou et tête
            corps.spawn((Mesh3d(cou), MeshMaterial3d(peau.clone()), Transform::from_xyz(0.0, 0.57, 0.0)));
            corps.spawn((
                Mesh3d(tete.clone()),
                MeshMaterial3d(peau.clone()),
                Transform::from_xyz(0.0, 0.73, 0.0).with_scale(Vec3::new(0.125, 0.145, 0.135)),
            ));
            // Cheveux courts : une calotte un peu plus grande, décalée vers le haut et l'arrière
            corps.spawn((
                Mesh3d(tete),
                MeshMaterial3d(cheveux),
                Transform::from_xyz(0.0, 0.775, 0.02).with_scale(Vec3::new(0.135, 0.115, 0.14)),
            ));

            // Jambes : pivot à la hanche, maillage pendu dessous
            for (membre, x) in [(Membre::JambeGauche, -0.10), (Membre::JambeDroite, 0.10)] {
                corps
                    .spawn((membre, Transform::from_xyz(x, HANCHES, 0.0), Visibility::default()))
                    .with_children(|pivot| {
                        pivot.spawn((
                            Mesh3d(jambe.clone()),
                            MeshMaterial3d(pantalon.clone()),
                            Transform::from_xyz(0.0, -0.40, 0.0),
                        ));
                        pivot.spawn((
                            Mesh3d(chaussure.clone()),
                            MeshMaterial3d(chaussures.clone()),
                            Transform::from_xyz(0.0, -0.755, -0.05),
                        ));
                    });
            }

            // Bras : pivot à l'épaule
            for (membre, x) in [(Membre::BrasGauche, -0.27), (Membre::BrasDroit, 0.27)] {
                corps
                    .spawn((membre, Transform::from_xyz(x, EPAULES, 0.0), Visibility::default()))
                    .with_children(|pivot| {
                        // Manche courte
                        pivot.spawn((
                            Mesh3d(boite.clone()),
                            MeshMaterial3d(t_shirt.clone()),
                            Transform::from_xyz(0.0, -0.06, 0.0).with_scale(Vec3::new(0.15, 0.16, 0.15)),
                        ));
                        pivot.spawn((
                            Mesh3d(bras.clone()),
                            MeshMaterial3d(peau.clone()),
                            Transform::from_xyz(0.0, -0.27, 0.0),
                        ));
                        pivot.spawn((
                            Mesh3d(main.clone()),
                            MeshMaterial3d(peau.clone()),
                            Transform::from_xyz(0.0, -0.55, 0.0),
                        ));
                    });
            }
        });
}

fn animer(
    time: Res<Time>,
    joueurs: Query<(&Joueur, &LinearVelocity)>,
    mut corps: Query<(&mut Transform, &mut Allure, &ChildOf), (With<Apparence>, Without<Membre>)>,
    mut membres: Query<(&mut Transform, &Membre), Without<Apparence>>,
) {
    let dt = time.delta_secs();
    let lissage = (DOUCEUR * dt).min(1.0);

    for (mut transform_corps, mut allure, parent) in &mut corps {
        let Ok((joueur, vitesse)) = joueurs.get(parent.parent()) else {
            continue;
        };
        let vitesse_sol = Vec2::new(vitesse.x, vitesse.z).length();
        let ratio = (vitesse_sol / 6.0).clamp(0.0, 1.0);

        if joueur.au_sol {
            // Le cycle de pas avance avec la distance parcourue
            allure.phase += vitesse_sol * dt * FREQUENCE_PAS * 0.5;
        }
        let balancier = allure.phase.sin();

        // Petit rebond à chaque pas
        let hauteur = if joueur.au_sol {
            REBOND * ratio * (allure.phase * 2.0).sin().abs()
        } else {
            0.0
        };
        transform_corps.translation.y += (hauteur - transform_corps.translation.y) * lissage;

        for (mut transform, membre) in &mut membres {
            let angle = if joueur.au_sol {
                match membre {
                    Membre::JambeGauche => AMPLITUDE_JAMBES * ratio * balancier,
                    Membre::JambeDroite => -AMPLITUDE_JAMBES * ratio * balancier,
                    Membre::BrasGauche => -AMPLITUDE_BRAS * ratio * balancier,
                    Membre::BrasDroit => AMPLITUDE_BRAS * ratio * balancier,
                }
            } else {
                // Pose de saut : une jambe en avant, bras légèrement levés
                match membre {
                    Membre::JambeGauche => 0.55,
                    Membre::JambeDroite => -0.25,
                    Membre::BrasGauche => -0.9,
                    Membre::BrasDroit => -0.6,
                }
            };
            // Bras un peu écartés du corps en l'air
            let ecart = match (membre, joueur.au_sol) {
                (Membre::BrasGauche, false) => -0.35,
                (Membre::BrasDroit, false) => 0.35,
                (Membre::BrasGauche, true) => -0.06,
                (Membre::BrasDroit, true) => 0.06,
                _ => 0.0,
            };
            // Rotation autour de X : angle positif = membre vers l'avant (-Z)
            let cible = Quat::from_rotation_z(ecart) * Quat::from_rotation_x(angle);
            transform.rotation = transform.rotation.slerp(cible, lissage);
        }
    }
}
