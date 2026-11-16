//! L'île en version brute : relief à facettes généré par le code, chemins,
//! et des volumes simples à l'emplacement de chaque projet.

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use crate::rendu::{
    hasard, matiere_mate, texture_bois, texture_grain, texture_pierre, texture_terre_cuite,
};

// --- Plan de l'île (x : est, z : sud ; unités : mètres) ---
pub const PLACE: Vec2 = Vec2::new(0.0, 8.0);
pub const COLLINE: Vec2 = Vec2::new(-20.0, -16.0); // Randolienne
pub const POTAGER: Vec2 = Vec2::new(23.0, 4.0); // Smart Plant Pot
pub const USINE: Vec2 = Vec2::new(16.0, -24.0); // Imprimante 3D
const PLAGE_DEPART: Vec2 = Vec2::new(0.0, 33.0);

const HAUTEUR_PLATEAU: f32 = 1.6;
const HAUTEUR_COLLINE: f32 = 8.0;
const RAYON_SOMMET: f32 = 3.5; // sommet plat de la colline
const RAYON_COLLINE: f32 = 19.0;
const LARGEUR_CHEMIN: f32 = 1.4;

// --- Maillage du terrain ---
const TAILLE_TERRAIN: f32 = 130.0;
const CASES: usize = 60; // 60 × 60 cases d'environ 2 m : les facettes se voient bien

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
fn distance_chemin(p: Vec2) -> f32 {
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
        0.25 * lissage(0.0, 4.0, vers_interieur) + (HAUTEUR_PLATEAU - 0.25) * lissage(5.0, 12.0, vers_interieur)
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
    let brique = images.add(texture_terre_cuite());
    let bois = images.add(texture_bois());
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
    let terre = uni(0.45, 0.30, 0.18);
    let feuillage = uni(0.22, 0.62, 0.20);
    let tronc = uni(0.55, 0.40, 0.25);
    let pot = uni(0.85, 0.45, 0.28);
    let toit = uni(0.40, 0.45, 0.52);
    let or = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.82, 0.15),
        unlit: true,
        ..default()
    });

    let mut texture = |image: &Handle<Image>, repetitions: Vec2| materials.add(matiere_mate(image.clone(), repetitions));
    let mat_pierre = texture(&pierre, Vec2::splat(1.0));
    let mat_brique = texture(&brique, Vec2::new(5.0, 3.0));
    let mat_bois = texture(&bois, Vec2::new(1.0, 6.0));
    let mat_bois_court = texture(&bois, Vec2::splat(1.0));

    // --- Ponton au sud ---
    pave(&mut commands, &mut meshes, &mat_bois, Vec3::new(3.0, 0.3, 17.0), Vec3::new(0.0, 0.2, 45.5), Quat::IDENTITY, true);
    for z in [40.0, 46.0, 52.0] {
        for x in [-1.3, 1.3] {
            pave(&mut commands, &mut meshes, &mat_bois_court, Vec3::new(0.35, 3.0, 0.35), Vec3::new(x, -0.9, z), Quat::IDENTITY, false);
        }
    }

    // --- Colline : emplacement de la Randolienne (mât + rotor provisoires) ---
    let sommet = sur_le_sol(COLLINE, 0.0);
    pave(&mut commands, &mut meshes, &mat_pierre, Vec3::new(1.6, 0.4, 1.6), sommet + Vec3::Y * 0.2, Quat::IDENTITY, true);
    pave(&mut commands, &mut meshes, &metal_sombre, Vec3::new(0.25, 3.2, 0.25), sommet + Vec3::Y * 2.0, Quat::IDENTITY, true);
    let rotor = commands
        .spawn((Rotation(2.2), Transform::from_translation(sommet + Vec3::Y * 4.5), Visibility::default()))
        .id();
    for i in 0..3 {
        let angle = i as f32 * std::f32::consts::TAU / 3.0;
        let pale = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(0.08, 1.8, 0.5))),
                MeshMaterial3d(metal.clone()),
                Transform::from_rotation(Quat::from_rotation_y(angle)).mul_transform(Transform::from_xyz(0.8, 0.0, 0.0)),
            ))
            .id();
        commands.entity(rotor).add_child(pale);
    }

    // --- Potager : emplacement du Smart Plant Pot ---
    let centre = sur_le_sol(POTAGER, 0.0);
    for i in 0..4 {
        let z = -3.0 + 2.0 * i as f32;
        pave(&mut commands, &mut meshes, &terre, Vec3::new(7.0, 0.3, 1.1), centre + Vec3::new(2.5, 0.1, z), Quat::IDENTITY, true);
        // Rangée de plants
        for j in 0..6 {
            pave(
                &mut commands,
                &mut meshes,
                &feuillage,
                Vec3::new(0.45, 0.45, 0.45),
                centre + Vec3::new(-0.3 + 1.1 * j as f32, 0.45, z),
                Quat::from_rotation_y(0.6 * (i + j) as f32),
                false,
            );
        }
    }
    // Clôture basse sur trois côtés
    pave(&mut commands, &mut meshes, &mat_bois_court, Vec3::new(9.0, 0.7, 0.15), centre + Vec3::new(2.5, 0.35, -4.4), Quat::IDENTITY, true);
    pave(&mut commands, &mut meshes, &mat_bois_court, Vec3::new(9.0, 0.7, 0.15), centre + Vec3::new(2.5, 0.35, 4.4), Quat::IDENTITY, true);
    pave(&mut commands, &mut meshes, &mat_bois_court, Vec3::new(0.15, 0.7, 8.8), centre + Vec3::new(7.0, 0.35, 0.0), Quat::IDENTITY, true);
    // Socle et pot provisoire
    let socle_pot = centre + Vec3::new(-3.5, 0.0, 0.0);
    pave(&mut commands, &mut meshes, &mat_pierre, Vec3::new(1.0, 0.8, 1.0), socle_pot + Vec3::Y * 0.4, Quat::IDENTITY, true);
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.5).mesh().resolution(8))),
        MeshMaterial3d(pot.clone()),
        Transform::from_translation(socle_pot + Vec3::Y * 1.05),
    ));
    pave(&mut commands, &mut meshes, &feuillage, Vec3::new(0.35, 0.4, 0.35), socle_pot + Vec3::Y * 1.45, Quat::from_rotation_y(0.5), false);

    // --- Usine : emplacement de l'imprimante 3D ---
    let base = sur_le_sol(USINE, 0.0);
    pave(&mut commands, &mut meshes, &mat_brique, Vec3::new(12.0, 5.0, 8.0), base + Vec3::new(0.0, 2.5, -2.0), Quat::IDENTITY, true);
    // Toit en dents de scie
    for i in 0..3 {
        pave(
            &mut commands,
            &mut meshes,
            &toit,
            Vec3::new(4.2, 0.25, 8.2),
            base + Vec3::new(-4.0 + 4.0 * i as f32, 5.6, -2.0),
            Quat::from_rotation_z(0.38),
            false,
        );
    }
    // Cheminée
    pave(&mut commands, &mut meshes, &mat_brique, Vec3::new(1.2, 9.0, 1.2), base + Vec3::new(4.5, 4.5, -4.5), Quat::IDENTITY, true);
    // Porte
    pave(&mut commands, &mut meshes, &metal_sombre, Vec3::new(2.4, 3.0, 0.2), base + Vec3::new(-2.0, 1.5, 2.05), Quat::IDENTITY, false);
    // Socle devant l'entrée, avec une « imprimante » provisoire
    let socle_usine = base + Vec3::new(2.5, 0.0, 5.0);
    pave(&mut commands, &mut meshes, &mat_pierre, Vec3::new(1.4, 0.6, 1.4), socle_usine + Vec3::Y * 0.3, Quat::IDENTITY, true);
    pave(&mut commands, &mut meshes, &metal, Vec3::new(0.8, 0.8, 0.8), socle_usine + Vec3::Y * 1.0, Quat::IDENTITY, false);

    // --- Repères dorés au-dessus de chaque projet ---
    let repere = meshes.add(Sphere::new(0.35).mesh().ico(0).unwrap());
    let emplacements = [sommet + Vec3::new(2.0, 2.2, 2.0), socle_pot + Vec3::Y * 2.6, socle_usine + Vec3::Y * 2.4];
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

    // --- Palmiers ---
    let palmiers = [
        Vec2::new(-8.0, 31.0),
        Vec2::new(9.0, 30.0),
        Vec2::new(27.0, 22.0),
        Vec2::new(34.0, 8.0),
        Vec2::new(33.0, -12.0),
        Vec2::new(4.0, -33.0),
        Vec2::new(-14.0, -36.0),
        Vec2::new(-36.0, 2.0),
        Vec2::new(-30.0, 18.0),
        Vec2::new(-20.0, 27.0),
        Vec2::new(8.0, 14.0),
        Vec2::new(-9.0, 2.0),
    ];
    let mesh_tronc = meshes.add(Cylinder::new(0.22, 5.0).mesh().resolution(6));
    let mesh_palme = meshes.add(Cuboid::new(0.9, 0.06, 2.6));
    for (i, p) in palmiers.into_iter().enumerate() {
        let pied = sur_le_sol(p, 0.0);
        let penche = Quat::from_rotation_y(i as f32 * 1.7) * Quat::from_rotation_z(0.12);
        commands
            .spawn((
                Mesh3d(mesh_tronc.clone()),
                MeshMaterial3d(tronc.clone()),
                Transform::from_translation(pied + Vec3::Y * 2.4).with_rotation(penche),
                RigidBody::Static,
                Collider::cylinder(0.25, 5.0),
            ))
            .with_children(|arbre| {
                for k in 0..6 {
                    let autour = Quat::from_rotation_y(k as f32 * std::f32::consts::TAU / 6.0);
                    arbre.spawn((
                        Mesh3d(mesh_palme.clone()),
                        MeshMaterial3d(feuillage.clone()),
                        Transform::from_translation(Vec3::Y * 2.5)
                            .with_rotation(autour * Quat::from_rotation_x(0.55))
                            .mul_transform(Transform::from_xyz(0.0, 0.0, 1.3)),
                    ));
                }
            });
    }
}

/// Pose un pavé ; `solide` décide si le personnage bute dessus.
fn pave(
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
        volume.insert((RigidBody::Static, Collider::cuboid(taille.x, taille.y, taille.z)));
    }
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

    // Grille de points
    let mut points = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            let x = -moitie + i as f32 * pas;
            let z = -moitie + j as f32 * pas;
            points.push(Vec3::new(x, hauteur(x, z), z));
        }
    }

    // Triangles. Pour l'affichage, chaque triangle a ses propres sommets :
    // une seule couleur et une seule normale par facette, donc des arêtes bien visibles.
    let sable = couleur(0.93, 0.82, 0.56);
    let sable_mouille = couleur(0.80, 0.72, 0.52);
    let herbe = couleur(0.33, 0.70, 0.26);
    let roche = couleur(0.56, 0.53, 0.48);
    let chemin = couleur(0.80, 0.64, 0.42);

    let mut triangles: Vec<[u32; 3]> = Vec::with_capacity(CASES * CASES * 2);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normales: Vec<[f32; 3]> = Vec::new();
    let mut couleurs: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for j in 0..CASES {
        for i in 0..CASES {
            let a = (j * n + i) as u32;
            let b = a + 1;
            let c = a + n as u32;
            let d = c + 1;
            for (numero, triangle) in [[a, c, b], [b, c, d]].into_iter().enumerate() {
                triangles.push(triangle);
                let [p0, p1, p2] = triangle.map(|k| points[k as usize]);
                let normale = (p1 - p0).cross(p2 - p0).normalize();
                let milieu = (p0 + p1 + p2) / 3.0;

                let base = if distance_chemin(milieu.xz()) < LARGEUR_CHEMIN && milieu.y > 0.5 {
                    chemin
                } else if normale.y < 0.80 {
                    roche
                } else if milieu.y < 0.0 {
                    sable_mouille
                } else if milieu.y < 0.9 {
                    sable
                } else {
                    herbe
                };
                // Légère variation d'une facette à l'autre
                let nuance = 0.92 + 0.16 * hasard(i as u32, j as u32, 10 + numero as u32);

                for p in [p0, p1, p2] {
                    positions.push(p.to_array());
                    normales.push(normale.to_array());
                    couleurs.push([base[0] * nuance, base[1] * nuance, base[2] * nuance, 1.0]);
                    uvs.push([p.x / 4.0, p.z / 4.0]);
                }
            }
        }
    }

    let maillage = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normales)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, couleurs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    commands.spawn((
        Mesh3d(meshes.add(maillage)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(texture_grain())),
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
