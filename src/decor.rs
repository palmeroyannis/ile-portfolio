//! Décor de l'île : écume sur le rivage, palmiers, herbe, fleurs, rochers et ponton.
//! Tout est fabriqué par le code, en facettes, et regroupé en quelques gros maillages
//! (un seul par type d'objet) pour rester léger dans le navigateur.

use std::f32::consts::{PI, TAU};

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use crate::geo::{matiere_couleurs, nuance, Constructeur};
use crate::monde::{
    distance_chemin, hauteur, rayon_cote, sur_le_sol, COLLINE, PLACE, PLAGE_DEPART, POTAGER, USINE,
};
use crate::rendu::{hasard, texture_grain, NIVEAU_MER};

// --- Réglages ---
const PAS_HERBE: f32 = 1.5; // espacement de la grille de semis, en mètres
const DENSITE_HERBE: f32 = 0.55; // proportion de cases qui reçoivent une touffe
const PART_FLEURS: f32 = 0.10;

// Modèle du palmier (assets/modeles/palmier.glb, 312 triangles) : tronc de 3 m qui se courbe vers +X.
const MODELE_PALMIER: &str = "modeles/palmier.glb";
const SOMMET_MODELE: Vec3 = Vec3::new(0.35, 3.0, 0.0); // haut du tronc dans le modèle
const RAYON_PIED_MODELE: f32 = 0.20;
const RAYON_HAUT_MODELE: f32 = 0.12;

const HERBE: Color = Color::srgb(0.38, 0.70, 0.22);
const PIERRE: Color = Color::srgb(0.58, 0.56, 0.52);
const BOIS: Color = Color::srgb(0.60, 0.43, 0.26);
const BOIS_SOMBRE: Color = Color::srgb(0.40, 0.28, 0.17);

/// Emplacements des palmiers (sur le plan de l'île).
const PALMIERS: [Vec2; 16] = [
    Vec2::new(-8.0, 31.0),
    Vec2::new(9.0, 30.5),
    Vec2::new(12.5, 33.0),
    Vec2::new(27.0, 22.0),
    Vec2::new(34.0, 8.0),
    Vec2::new(35.0, -9.0),
    Vec2::new(30.0, -26.0),
    Vec2::new(4.0, -34.0),
    Vec2::new(-14.0, -36.0),
    Vec2::new(-36.0, 2.0),
    Vec2::new(-34.0, -4.0),
    Vec2::new(-30.0, 18.0),
    Vec2::new(-20.0, 27.0),
    Vec2::new(-23.0, 30.0),
    Vec2::new(8.0, 15.0),
    Vec2::new(-9.0, 4.0),
];

pub struct DecorPlugin;

impl Plugin for DecorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_decor)
            .add_systems(Update, animer_ecume);
    }
}

/// Bande d'écume le long du rivage, qui va et vient.
#[derive(Component)]
struct Ecume {
    /// Vague qui arrive du large (sinon : écume qui lèche le sable)
    vague: bool,
    decalage: f32,
}

fn creer_decor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let grain = images.add(texture_grain());
    let simple = materials.add(matiere_couleurs(grain.clone(), false));
    let double = materials.add(matiere_couleurs(grain, true));

    creer_ecume(&mut commands, &mut meshes, &mut materials);
    creer_palmiers(&mut commands, &mut meshes, &asset_server, &simple);
    creer_herbe(&mut commands, &mut meshes, &double);
    creer_rochers(&mut commands, &mut meshes, &simple);
    creer_ponton(&mut commands, &mut meshes, &simple);
}

// ---------------------------------------------------------------------------
// Écume
// ---------------------------------------------------------------------------

/// Anneau qui suit la côte : de `interieur` à `exterieur` mètres de la ligne de rivage,
/// avec une opacité qui varie en travers de la bande (une valeur par rangée de sommets).
fn anneau_cote(cote: &[f32], decalages: &[f32], opacites: &[f32]) -> Mesh {
    let n = cote.len();
    let rangees = decalages.len();
    let mut positions = Vec::new();
    let mut couleurs = Vec::new();
    let mut normales = Vec::new();
    let mut uvs = Vec::new();
    for (i, r) in cote.iter().enumerate() {
        let angle = i as f32 / n as f32 * TAU;
        let direction = Vec2::new(angle.cos(), angle.sin());
        for k in 0..rangees {
            let p = direction * (r + decalages[k]);
            positions.push([p.x, 0.0, p.y]);
            couleurs.push([1.0, 1.0, 1.0, opacites[k]]);
            normales.push([0.0, 1.0, 0.0]);
            uvs.push([i as f32, k as f32]);
        }
    }
    let mut indices = Vec::new();
    for i in 0..n {
        let suivant = (i + 1) % n;
        for k in 0..rangees - 1 {
            let a = (i * rangees + k) as u32;
            let b = (suivant * rangees + k) as u32;
            let c = (suivant * rangees + k + 1) as u32;
            let d = (i * rangees + k + 1) as u32;
            // Face tournée vers le haut, visible depuis la caméra
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normales)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, couleurs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

fn creer_ecume(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let cote: Vec<f32> = (0..240)
        .map(|i| rayon_cote(i as f32 / 240.0 * TAU))
        .collect();

    let matiere = |materials: &mut Assets<StandardMaterial>, biais: f32| {
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            // Dessinée après la mer, qui est transparente elle aussi
            depth_bias: biais,
            ..default()
        })
    };

    // Écume collée au sable
    commands.spawn((
        Ecume {
            vague: false,
            decalage: 0.0,
        },
        Mesh3d(meshes.add(anneau_cote(
            &cote,
            &[-0.6, 0.15, 0.7, 1.8],
            &[0.9, 0.85, 0.45, 0.0],
        ))),
        MeshMaterial3d(matiere(materials, 4.0)),
        Transform::from_xyz(0.0, NIVEAU_MER + 0.02, 0.0),
    ));

    // Deux vagues qui arrivent du large, décalées dans le temps
    let mesh_vague = meshes.add(anneau_cote(&cote, &[0.0, 0.35, 0.9], &[0.0, 0.8, 0.0]));
    for decalage in [0.0, 0.5] {
        commands.spawn((
            Ecume {
                vague: true,
                decalage,
            },
            Mesh3d(mesh_vague.clone()),
            MeshMaterial3d(matiere(materials, 5.0)),
            Transform::from_xyz(0.0, NIVEAU_MER + 0.025, 0.0),
        ));
    }
}

fn animer_ecume(
    time: Res<Time>,
    mut ecumes: Query<(&Ecume, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    for (ecume, mut transform, matiere) in &mut ecumes {
        let (echelle, opacite) = if ecume.vague {
            // Part à ~3 m du rivage et vient mourir sur le sable en 4 secondes
            let cycle = (t / 4.0 + ecume.decalage).fract();
            (1.0 + 0.07 * (1.0 - cycle), (PI * cycle).sin() * 0.8)
        } else {
            (1.0 + 0.006 * (t * 1.4).sin(), 0.85 + 0.15 * (t * 1.4).sin())
        };
        transform.scale = Vec3::new(echelle, 1.0, echelle);
        if let Some(mut m) = materials.get_mut(&matiere.0) {
            m.base_color = Color::srgba(1.0, 1.0, 1.0, opacite);
        }
    }
}

// ---------------------------------------------------------------------------
// Palmiers
// ---------------------------------------------------------------------------

/// Forme d'un palmier, partagée avec le calcul des ombres peintes au sol.
pub struct Palmier {
    pub pied: Vec3,
    pub hauteur_tronc: f32,
    /// Déport horizontal du sommet par rapport au pied
    pub penche: Vec3,
}

impl Palmier {
    pub fn sommet(&self) -> Vec3 {
        self.pied + Vec3::Y * self.hauteur_tronc + self.penche
    }
}

pub fn palmiers() -> Vec<Palmier> {
    PALMIERS
        .iter()
        .enumerate()
        .map(|(n, p)| {
            let graine = n as u32;
            // Les palmiers penchent vers l'extérieur de l'île, comme poussés par le vent du large
            let dehors = p.normalize_or(Vec2::X);
            let lateral = Vec2::new(-dehors.y, dehors.x) * (hasard(graine, 2, 20) - 0.5);
            let penche = (dehors + lateral).normalize() * (0.9 + 0.9 * hasard(graine, 3, 20));
            Palmier {
                pied: sur_le_sol(*p, -0.1),
                hauteur_tronc: 4.6 + 1.6 * hasard(graine, 1, 20),
                penche: Vec3::new(penche.x, 0.0, penche.y),
            }
        })
        .collect()
}

fn creer_palmiers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    simple: &Handle<StandardMaterial>,
) {
    // Un seul chargement, réutilisé par tous les palmiers : Bevy les dessine ensemble
    let modele = asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODELE_PALMIER));
    let mut noix = Constructeur::default();

    for (n, palmier) in palmiers().iter().enumerate() {
        let (pied, penche) = (palmier.pied, palmier.penche);
        let sommet = palmier.sommet();

        // On oriente et on agrandit le modèle pour que le haut de son tronc tombe pile sur
        // le sommet prévu : même inclinaison vers le large, ombres peintes inchangées.
        let voulu = sommet - pied;
        let dehors = Vec2::new(penche.x, penche.z).normalize_or(Vec2::X);
        let lacet = Quat::from_rotation_y((-dehors.y).atan2(dehors.x));
        let inclinaison =
            Quat::from_rotation_arc((lacet * SOMMET_MODELE).normalize(), voulu.normalize());
        let echelle = voulu.length() / SOMMET_MODELE.length();
        commands.spawn((
            WorldAssetRoot(modele.clone()),
            Transform::from_translation(pied)
                .with_rotation(inclinaison * lacet)
                .with_scale(Vec3::splat(echelle)),
        ));

        // Noix de coco, collées au haut du tronc
        let rayon = RAYON_HAUT_MODELE * echelle + 0.06;
        for k in 0..3 {
            let a = k as f32 * TAU / 3.0 + n as f32;
            noix.losange(
                sommet + Vec3::new(a.cos() * rayon, -0.15, a.sin() * rayon),
                Vec3::splat(0.13),
                Color::srgb(0.42, 0.30, 0.15),
            );
        }

        // Collision : un poteau sur le bas du tronc, là où le personnage peut buter
        commands.spawn((
            Transform::from_translation(pied + Vec3::Y * 1.5 + penche * 0.05),
            RigidBody::Static,
            Collider::cylinder(RAYON_PIED_MODELE * echelle, 3.0),
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(noix.construire())),
        MeshMaterial3d(simple.clone()),
    ));
}

// ---------------------------------------------------------------------------
// Herbe et fleurs
// ---------------------------------------------------------------------------

/// Vrai si l'endroit est dégagé : ni chemin, ni lieu de projet, ni place.
fn endroit_libre(p: Vec2, marge_chemin: f32) -> bool {
    distance_chemin(p) > marge_chemin
        && p.distance(PLACE) > 5.0
        && p.distance(POTAGER + Vec2::new(1.5, 0.0)) > 7.5
        && p.distance(USINE + Vec2::new(0.0, 0.0)) > 10.0
        && p.distance(COLLINE) > 4.5
        && p.distance(PLAGE_DEPART) > 4.0
}

fn pente(p: Vec2) -> f32 {
    let e = 0.5;
    let dx = hauteur(p.x + e, p.y) - hauteur(p.x - e, p.y);
    let dz = hauteur(p.x, p.y + e) - hauteur(p.x, p.y - e);
    Vec2::new(dx, dz).length() / (2.0 * e)
}

fn creer_herbe(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    double: &Handle<StandardMaterial>,
) {
    let mut herbe = Constructeur::default();
    let couleurs_fleurs = [
        Color::srgb(1.0, 1.0, 0.95),
        Color::srgb(1.0, 0.85, 0.15),
        Color::srgb(0.95, 0.45, 0.65),
        Color::srgb(0.90, 0.25, 0.20),
        Color::srgb(0.65, 0.55, 0.95),
    ];

    let occultants = crate::ombres::occultants();
    let etendue = (46.0 / PAS_HERBE) as i32;
    for j in -etendue..=etendue {
        for i in -etendue..=etendue {
            let (iu, ju) = ((i + 1000) as u32, (j + 1000) as u32);
            if hasard(iu, ju, 30) > DENSITE_HERBE {
                continue;
            }
            let p = Vec2::new(
                (i as f32 + hasard(iu, ju, 31) - 0.5) * PAS_HERBE,
                (j as f32 + hasard(iu, ju, 32) - 0.5) * PAS_HERBE,
            );
            let h = hauteur(p.x, p.y);
            // Seulement sur l'herbe (pas le sable), pas dans les pentes rocheuses
            if h < 1.0 || pente(p) > 0.55 || !endroit_libre(p, 1.8) {
                continue;
            }
            let pied = Vec3::new(p.x, h - 0.03, p.y);
            let sol = Vec3::new(p.x, h, p.y);
            let ombre = crate::ombres::ombre(sol, &occultants);
            let occlusion = crate::ombres::occlusion(sol, &occultants);
            let ombrer = |c: Color| crate::ombres::ombrer_couleur(c, ombre, occlusion);

            if hasard(iu, ju, 33) < PART_FLEURS {
                // Fleur : une tige et une corolle en losange
                let haut = pied + Vec3::Y * (0.30 + 0.2 * hasard(iu, ju, 34));
                herbe.triangle_double(
                    pied + Vec3::X * 0.02,
                    pied - Vec3::X * 0.02,
                    haut,
                    ombrer(HERBE),
                );
                let c = couleurs_fleurs[(hasard(iu, ju, 35) * 4.99) as usize];
                herbe.losange(haut, Vec3::new(0.09, 0.05, 0.09), ombrer(c));
                continue;
            }

            // Touffe : trois ou quatre brins pointus, inclinés dans des directions différentes
            let brins = 3 + (hasard(iu, ju, 36) * 1.99) as u32;
            for b in 0..brins {
                let angle = b as f32 / brins as f32 * TAU + hasard(iu, ju, 37 + b) * 1.2;
                let d = Vec3::new(angle.cos(), 0.0, angle.sin());
                let travers = Vec3::new(-d.z, 0.0, d.x) * 0.07;
                let hauteur_brin = 0.30 + 0.30 * hasard(iu, ju, 40 + b);
                let pointe = pied + Vec3::Y * hauteur_brin + d * 0.18;
                let teinte = ombrer(nuance(HERBE, 0.85 + 0.35 * hasard(iu, ju, 45 + b)));
                herbe.triangle_double(pied + travers, pied - travers, pointe, teinte);
            }
        }
    }

    commands.spawn((
        Mesh3d(meshes.add(herbe.construire())),
        MeshMaterial3d(double.clone()),
    ));
}

// ---------------------------------------------------------------------------
// Rochers
// ---------------------------------------------------------------------------

/// Emplacement, taille et graine de chaque rocher (partagé avec les ombres peintes au sol).
pub fn rochers() -> Vec<(Vec3, Vec3, u32)> {
    let mut liste = Vec::new();
    let mut graine = 100;

    // Autour de l'île, à cheval sur le rivage (certains dans l'eau)
    for k in 0..26 {
        let angle = k as f32 / 26.0 * TAU + hasard(k, 0, 51) * 0.18;
        // Pas devant le ponton ni sur la plage de départ (au sud, angle ≈ π/2)
        let ecart = (angle - PI / 2.0).rem_euclid(TAU);
        if ecart < 0.45 || ecart > TAU - 0.45 {
            continue;
        }
        let r = rayon_cote(angle) + (hasard(k, 1, 51) * 7.0 - 2.5);
        let p = Vec2::new(angle.cos(), angle.sin()) * r;
        let taille = 0.6 + 1.4 * hasard(k, 2, 51);
        let centre = Vec3::new(p.x, hauteur(p.x, p.y) + taille * 0.25, p.y);
        liste.push((
            centre,
            Vec3::new(taille * 1.2, taille * 0.8, taille),
            graine,
        ));
        graine += 1;
        // Parfois un petit rocher à côté
        if hasard(k, 3, 51) > 0.5 {
            let q = p + Vec2::new(hasard(k, 4, 51) - 0.5, hasard(k, 5, 51) - 0.5) * 3.0;
            let t = taille * 0.45;
            liste.push((
                Vec3::new(q.x, hauteur(q.x, q.y) + t * 0.2, q.y),
                Vec3::splat(t),
                graine,
            ));
            graine += 1;
        }
    }

    // Sur les flancs et au bord du sommet de la colline
    for k in 0..9 {
        let angle = k as f32 / 9.0 * TAU + 0.3;
        let r = if k % 3 == 0 {
            4.6
        } else {
            7.0 + 5.0 * hasard(k, 6, 52)
        };
        let p = COLLINE + Vec2::new(angle.cos(), angle.sin()) * r;
        if !endroit_libre(p, 2.0) && p.distance(COLLINE) > 4.5 {
            continue;
        }
        let taille = 0.5 + 0.9 * hasard(k, 7, 52);
        let centre = Vec3::new(p.x, hauteur(p.x, p.y) + taille * 0.2, p.y);
        liste.push((
            centre,
            Vec3::new(taille, taille * 0.7, taille * 1.1),
            graine,
        ));
        graine += 1;
    }
    liste
}

fn creer_rochers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    simple: &Handle<StandardMaterial>,
) {
    let mut maillage = Constructeur::default();
    for (centre, taille, graine) in rochers() {
        let rotation = Quat::from_rotation_y(hasard(graine, 0, 50) * TAU)
            * Quat::from_rotation_x(hasard(graine, 1, 50) * 0.4);
        let teinte = nuance(PIERRE, 0.85 + 0.25 * hasard(graine, 2, 50));
        let points = maillage.rocher(centre, taille, rotation, graine, teinte);
        if let Some(collision) = Collider::convex_hull(points) {
            commands.spawn((Transform::default(), RigidBody::Static, collision));
        }
    }
    maillage.assombrir_pres_du_sol();
    commands.spawn((
        Mesh3d(meshes.add(maillage.construire())),
        MeshMaterial3d(simple.clone()),
    ));
}

// ---------------------------------------------------------------------------
// Ponton
// ---------------------------------------------------------------------------

fn creer_ponton(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    simple: &Handle<StandardMaterial>,
) {
    let mut bois = Constructeur::default();
    let (debut, fin) = (37.0_f32, 54.0_f32);
    let dessus = 0.35;

    // Planches en travers, avec un petit jour entre elles et des teintes variées
    let mut z = debut;
    let mut k = 0;
    while z < fin {
        let teinte = nuance(BOIS, 0.85 + 0.3 * hasard(k, 0, 60));
        let decalage = (hasard(k, 1, 60) - 0.5) * 0.08;
        bois.boite(
            Vec3::new(decalage, dessus - 0.04, z + 0.15),
            Vec3::new(3.0, 0.08, 0.28),
            Quat::from_rotation_y((hasard(k, 2, 60) - 0.5) * 0.03),
            teinte,
        );
        z += 0.33;
        k += 1;
    }
    // Longerons sous les planches
    for x in [-1.2, 0.0, 1.2] {
        bois.boite(
            Vec3::new(x, dessus - 0.16, (debut + fin) / 2.0),
            Vec3::new(0.16, 0.16, fin - debut),
            Quat::IDENTITY,
            BOIS_SOMBRE,
        );
    }
    // Pieux jusqu'au fond, et deux poteaux d'amarrage au bout
    let mut z = debut + 1.0;
    while z < fin {
        for x in [-1.4, 1.4] {
            let fond = hauteur(x, z) - 0.2;
            bois.cylindre(
                Vec3::new(x, fond, z),
                Vec3::new(x, dessus - 0.1, z),
                0.14,
                0.14,
                6,
                BOIS_SOMBRE,
                true,
            );
        }
        z += 4.0;
    }
    for x in [-1.4, 1.4] {
        bois.cylindre(
            Vec3::new(x, dessus - 0.1, fin - 0.3),
            Vec3::new(x, dessus + 0.7, fin - 0.3),
            0.13,
            0.12,
            6,
            BOIS,
            true,
        );
    }

    commands.spawn((
        Mesh3d(meshes.add(bois.construire())),
        MeshMaterial3d(simple.clone()),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, dessus - 0.15, (debut + fin) / 2.0),
        RigidBody::Static,
        Collider::cuboid(3.0, 0.3, fin - debut),
    ));
}
