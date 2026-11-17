//! L'île en version brute : relief à facettes généré par le code, chemins,
//! et des volumes simples à l'emplacement de chaque projet.

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::rendu::{hasard, matiere_mate, texture_grain, texture_pierre, texture_sol};

// --- Plan de l'île (x : est, z : sud ; unités : mètres) ---
pub const PLACE: Vec2 = Vec2::new(0.0, 8.0);
pub const COLLINE: Vec2 = Vec2::new(-20.0, -16.0); // Randolienne
pub const POTAGER: Vec2 = Vec2::new(23.0, 4.0); // Smart Plant Pot
pub const USINE: Vec2 = Vec2::new(16.0, -24.0); // Imprimante 3D
pub const PLAGE_DEPART: Vec2 = Vec2::new(0.0, 33.0);

const HAUTEUR_PLATEAU: f32 = 1.6;
const HAUTEUR_COLLINE: f32 = 8.0;
pub const RAYON_SOMMET: f32 = 3.5; // sommet plat de la colline
const RAYON_COLLINE: f32 = 19.0;
pub const LARGEUR_CHEMIN: f32 = 1.4;

// --- Maillage du terrain ---
const TAILLE_TERRAIN: f32 = 130.0;
const CASES: usize = 130; // cases d'un mètre : côte et chemins aux contours réguliers

pub struct MondePlugin;

impl Plugin for MondePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_monde)
            .add_systems(Update, animer_decor);
    }
}

/// Objets qui tournent sur eux-mêmes (rotor, repères).
#[derive(Component)]
struct Rotation(f32);

/// Repère doré flottant au-dessus d'un projet : s'en approcher permet d'ouvrir sa fiche.
#[derive(Component)]
pub struct Repere {
    hauteur_base: f32,
    /// Numéro du projet dans `projets::PROJETS`
    pub projet: usize,
}

/// Point d'apparition du personnage.
pub fn depart() -> Vec3 {
    sur_le_sol(PLAGE_DEPART, 1.2)
}

/// Position 3D d'un point du plan, posé sur le terrain (plus un décalage vertical).
pub fn sur_le_sol(p: Vec2, decalage: f32) -> Vec3 {
    Vec3::new(p.x, hauteur(p.x, p.y) + decalage, p.y)
}

// ---------------------------------------------------------------------------
// Forme de l'île
// ---------------------------------------------------------------------------

fn lissage(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Bruit doux entre 0 et 1 (valeurs aléatoires sur une grille, interpolées).
fn bruit(x: f32, z: f32) -> f32 {
    let (xi, zi) = (x.floor(), z.floor());
    let (fx, fz) = (x - xi, z - zi);
    let (fx, fz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let h = |dx: f32, dz: f32| hasard((xi + dx + 1000.0) as u32, (zi + dz + 1000.0) as u32, 42);
    let haut = h(0.0, 0.0) + (h(1.0, 0.0) - h(0.0, 0.0)) * fx;
    let bas = h(0.0, 1.0) + (h(1.0, 1.0) - h(0.0, 1.0)) * fx;
    haut + (bas - haut) * fz
}

fn distance_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Distance au chemin le plus proche.
pub fn distance_chemin(p: Vec2) -> f32 {
    let pied_colline = COLLINE + (PLACE - COLLINE).normalize() * (RAYON_COLLINE - 2.0);
    [
        (PLAGE_DEPART, PLACE),
        (PLACE, pied_colline),
        (pied_colline, COLLINE),
        (PLACE, POTAGER),
        (PLACE, USINE),
    ]
    .into_iter()
    .map(|(a, b)| distance_segment(p, a, b))
    .fold(f32::MAX, f32::min)
}

/// Hauteur du terrain en un point. Le niveau de la mer est légèrement sous 0.
pub fn hauteur(x: f32, z: f32) -> f32 {
    let p = Vec2::new(x, z);
    let r = p.length();
    let angle = z.atan2(x);

    // Côte irrégulière
    let rayon_cote = 43.0 + 4.5 * (3.0 * angle + 1.0).sin() + 2.5 * (5.0 * angle + 0.4).sin();
    let vers_interieur = rayon_cote - r;

    let mut h = if vers_interieur > 0.0 {
        // Plage en pente douce, puis plateau
        0.25 * lissage(0.0, 4.0, vers_interieur)
            + (HAUTEUR_PLATEAU - 0.25) * lissage(5.0, 12.0, vers_interieur)
    } else {
        // Fond marin
        (vers_interieur * 0.3).max(-3.0)
    };

    // Petites bosses sur le plateau, sauf sur les chemins et les lieux
    let pres_d_un_lieu = [PLACE, POTAGER, USINE]
        .into_iter()
        .map(|lieu| p.distance(lieu))
        .fold(f32::MAX, f32::min);
    let calme = lissage(1.5, 4.0, distance_chemin(p)).min(lissage(11.0, 15.0, pres_d_un_lieu));
    h += (bruit(x * 0.12, z * 0.12) - 0.5) * 0.9 * calme * lissage(8.0, 14.0, vers_interieur);

    // Colline à sommet plat
    let d = p.distance(COLLINE);
    h += HAUTEUR_COLLINE * (1.0 - lissage(RAYON_SOMMET, RAYON_COLLINE, d));

    h
}

/// Distance au centre de l'île où le terrain passe sous la surface de la mer, dans une direction donnée.
pub fn rayon_cote(angle: f32) -> f32 {
    let direction = Vec2::new(angle.cos(), angle.sin());
    let mut r = 30.0;
    while r < 64.0 {
        let p = direction * r;
        if hauteur(p.x, p.y) < crate::rendu::NIVEAU_MER {
            return r;
        }
        r += 0.1;
    }
    r
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

fn couleur(r: f32, g: f32, b: f32) -> [f32; 3] {
    let c = Color::srgb(r, g, b).to_linear();
    [c.red, c.green, c.blue]
}

fn creer_monde(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    creer_terrain(&mut commands, &mut meshes, &mut images, &mut materials);

    let pierre = images.add(texture_pierre());
    let grain = images.add(texture_grain());

    let mut uni = |r: f32, g: f32, b: f32| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(r, g, b),
            base_color_texture: Some(grain.clone()),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        })
    };
    let metal = uni(0.80, 0.82, 0.85);
    let metal_sombre = uni(0.35, 0.38, 0.42);
    let feuillage = uni(0.22, 0.62, 0.20);
    let pot = uni(0.85, 0.45, 0.28);
    let or = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.82, 0.15),
        unlit: true,
        ..default()
    });

    let mut texture = |image: &Handle<Image>, repetitions: Vec2| {
        materials.add(matiere_mate(image.clone(), repetitions))
    };
    let mat_pierre = texture(&pierre, Vec2::splat(1.0));

    // --- Colline : emplacement de la Randolienne (mât + rotor provisoires) ---
    let sommet = sur_le_sol(COLLINE, 0.0);
    pave(
        &mut commands,
        &mut meshes,
        &mat_pierre,
        Vec3::new(1.6, 0.4, 1.6),
        sommet + Vec3::Y * 0.2,
        Quat::IDENTITY,
        true,
    );
    pave(
        &mut commands,
        &mut meshes,
        &metal_sombre,
        Vec3::new(0.25, 3.2, 0.25),
        sommet + Vec3::Y * 2.0,
        Quat::IDENTITY,
        true,
    );
    let rotor = commands
        .spawn((
            Rotation(2.2),
            Transform::from_translation(sommet + Vec3::Y * 4.5),
            Visibility::default(),
        ))
        .id();
    for i in 0..3 {
        let angle = i as f32 * std::f32::consts::TAU / 3.0;
        let pale = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(0.08, 1.8, 0.5))),
                MeshMaterial3d(metal.clone()),
                Transform::from_rotation(Quat::from_rotation_y(angle))
                    .mul_transform(Transform::from_xyz(0.8, 0.0, 0.0)),
            ))
            .id();
        commands.entity(rotor).add_child(pale);
    }

    // --- Potager : socle et pot provisoires du Smart Plant Pot ---
    let centre = sur_le_sol(POTAGER, 0.0);
    // Socle et pot provisoire
    let socle_pot = centre + Vec3::new(-3.5, 0.0, 0.0);
    pave(
        &mut commands,
        &mut meshes,
        &mat_pierre,
        Vec3::new(1.0, 0.8, 1.0),
        socle_pot + Vec3::Y * 0.4,
        Quat::IDENTITY,
        true,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.5).mesh().resolution(8))),
        MeshMaterial3d(pot.clone()),
        Transform::from_translation(socle_pot + Vec3::Y * 1.05),
    ));
    pave(
        &mut commands,
        &mut meshes,
        &feuillage,
        Vec3::new(0.35, 0.4, 0.35),
        socle_pot + Vec3::Y * 1.45,
        Quat::from_rotation_y(0.5),
        false,
    );

    // --- Usine : socle et « imprimante » provisoires devant l'entrée ---
    let base = sur_le_sol(USINE, 0.0);
    // Socle devant l'entrée, avec une « imprimante » provisoire
    let socle_usine = base + Vec3::new(2.5, 0.0, 5.0);
    pave(
        &mut commands,
        &mut meshes,
        &mat_pierre,
        Vec3::new(1.4, 0.6, 1.4),
        socle_usine + Vec3::Y * 0.3,
        Quat::IDENTITY,
        true,
    );
    pave(
        &mut commands,
        &mut meshes,
        &metal,
        Vec3::new(0.8, 0.8, 0.8),
        socle_usine + Vec3::Y * 1.0,
        Quat::IDENTITY,
        false,
    );

    // --- Repères dorés au-dessus de chaque projet ---
    let repere = meshes.add(Sphere::new(0.35).mesh().ico(0).unwrap());
    let emplacements = [
        sommet + Vec3::new(2.0, 2.2, 2.0),
        socle_pot + Vec3::Y * 2.6,
        socle_usine + Vec3::Y * 2.4,
    ];
    for (projet, position) in emplacements.into_iter().enumerate() {
        commands.spawn((
            Repere {
                hauteur_base: position.y,
                projet,
            },
            Rotation(1.5),
            Mesh3d(repere.clone()),
            MeshMaterial3d(or.clone()),
            Transform::from_translation(position).with_scale(Vec3::new(1.0, 1.5, 1.0)),
        ));
    }
}

/// Pose un pavé ; `solide` décide si le personnage bute dessus.
pub fn pave(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    matiere: &Handle<StandardMaterial>,
    taille: Vec3,
    position: Vec3,
    rotation: Quat,
    solide: bool,
) {
    let mut volume = commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(taille))),
        MeshMaterial3d(matiere.clone()),
        Transform::from_translation(position).with_rotation(rotation),
    ));
    if solide {
        volume.insert((
            RigidBody::Static,
            Collider::cuboid(taille.x, taille.y, taille.z),
        ));
    }
}

/// Couleur du fond sous l'eau : sable clair près du bord, vert d'eau puis bleu profond.
fn fond_marin(y: f32) -> [f32; 3] {
    let profondeur = ((crate::rendu::NIVEAU_MER - y) / 2.6).clamp(0.0, 1.0);
    let clair = Vec3::new(0.86, 0.80, 0.60);
    let moyen = Vec3::new(0.52, 0.72, 0.62);
    let fonce = Vec3::new(0.16, 0.38, 0.50);
    let c = if profondeur < 0.4 {
        clair.lerp(moyen, profondeur / 0.4)
    } else {
        moyen.lerp(fonce, (profondeur - 0.4) / 0.6)
    };
    couleur(c.x, c.y, c.z)
}

/// Couleur du sol en un point : mélange doux entre sable, herbe, roche et chemin.
fn couleur_sol(x: f32, z: f32, h: f32, normale: Vec3) -> [f32; 3] {
    let v = |c: [f32; 3]| Vec3::from_array(c);
    if h < crate::rendu::NIVEAU_MER {
        return fond_marin(h);
    }
    let sable = v(couleur(0.93, 0.82, 0.56));
    let sable_mouille = v(couleur(0.80, 0.71, 0.50));
    let roche = v(couleur(0.56, 0.53, 0.48));
    let chemin = v(couleur(0.78, 0.62, 0.40));
    // Deux verts qui alternent en grandes taches, pour casser l'uniformité
    let tache = bruit(x * 0.08 + 7.0, z * 0.08 - 3.0);
    let herbe = v(couleur(0.30, 0.66, 0.22)).lerp(v(couleur(0.40, 0.70, 0.26)), tache);

    let mut c = sable_mouille.lerp(sable, lissage(crate::rendu::NIVEAU_MER, 0.2, h));
    c = c.lerp(herbe, lissage(0.75, 1.15, h));
    c = c.lerp(roche, lissage(0.86, 0.70, normale.y));
    if h > 0.5 {
        let d = distance_chemin(Vec2::new(x, z));
        c = c.lerp(
            chemin,
            1.0 - lissage(LARGEUR_CHEMIN - 0.35, LARGEUR_CHEMIN + 0.35, d),
        );
    }
    // Légères variations de luminosité, douces
    let variation = 0.94 + 0.12 * bruit(x * 0.3 - 11.0, z * 0.3 + 5.0);
    (c * variation).to_array()
}

fn creer_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) {
    let n = CASES + 1;
    let pas = TAILLE_TERRAIN / CASES as f32;
    let moitie = TAILLE_TERRAIN / 2.0;

    // Grille de points partagés : l'éclairage et les couleurs sont lissés d'un point à l'autre,
    // on ne voit plus les triangles du sol.
    let mut points = Vec::with_capacity(n * n);
    let mut normales = Vec::with_capacity(n * n);
    let mut couleurs = Vec::with_capacity(n * n);
    let mut uvs = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            let x = -moitie + i as f32 * pas;
            let z = -moitie + j as f32 * pas;
            let h = hauteur(x, z);
            let e = 0.5;
            let normale = Vec3::new(
                hauteur(x - e, z) - hauteur(x + e, z),
                2.0 * e,
                hauteur(x, z - e) - hauteur(x, z + e),
            )
            .normalize();
            let c = couleur_sol(x, z, h, normale);
            points.push(Vec3::new(x, h, z));
            normales.push(normale.to_array());
            couleurs.push([c[0], c[1], c[2], 1.0]);
            uvs.push([x / 3.0, z / 3.0]);
        }
    }

    let mut triangles: Vec<[u32; 3]> = Vec::with_capacity(CASES * CASES * 2);
    for j in 0..CASES {
        for i in 0..CASES {
            let a = (j * n + i) as u32;
            let b = a + 1;
            let c = a + n as u32;
            let d = c + 1;
            triangles.push([a, c, b]);
            triangles.push([b, c, d]);
        }
    }

    let maillage = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        points.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normales)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, couleurs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(triangles.iter().flatten().copied().collect()));

    commands.spawn((
        Mesh3d(meshes.add(maillage)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(texture_sol())),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        })),
        Transform::default(),
        RigidBody::Static,
        Collider::trimesh(points, triangles),
    ));
}

fn animer_decor(
    time: Res<Time>,
    mut tournants: Query<(&mut Transform, &Rotation, Option<&Repere>)>,
) {
    for (mut transform, rotation, repere) in &mut tournants {
        transform.rotate_y(rotation.0 * time.delta_secs());
        if let Some(repere) = repere {
            transform.translation.y = repere.hauteur_base + 0.2 * (time.elapsed_secs() * 2.0).sin();
        }
    }
}
