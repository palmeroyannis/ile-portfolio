//! Rendu « GameCube » : ciel en dégradé, mer qui défile, textures basse résolution,
//! éclairage simple sans reflets.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// --- Réglages ---
pub const COULEUR_HORIZON: Color = Color::srgb(0.82, 0.92, 0.97);
pub const NIVEAU_MER: f32 = -0.35;
const METRES_PAR_MOTIF_MER: f32 = 6.0; // taille d'un motif de la texture d'eau
/// Couleur et opacité de la mer selon la distance au centre de l'île :
/// transparente et claire au bord, profonde et opaque au large.
const DEGRADE_MER: [(f32, [f32; 3], f32); 7] = [
    (0.0, [0.16, 1.0, 0.90], 0.58),
    (42.0, [0.14, 0.98, 0.90], 0.60),
    (52.0, [0.08, 0.90, 0.90], 0.70),
    (70.0, [0.05, 0.74, 0.86], 0.83),
    (120.0, [0.04, 0.56, 0.82], 0.93),
    (260.0, [0.04, 0.40, 0.74], 1.0),
    (2000.0, [0.03, 0.34, 0.70], 1.0),
];
const VITESSE_MER: Vec2 = Vec2::new(0.035, 0.02); // défilement, en répétitions par seconde

/// Direction vers le soleil (unitaire) : au sud-ouest, à 43° au-dessus de l'horizon.
/// En marchant de la plage vers l'intérieur de l'île (vers le nord), on a le soleil dans le dos :
/// les façades qu'on regarde sont éclairées. On le voit en tournant la caméra vers la mer.
pub fn direction_soleil() -> Vec3 {
    Vec3::new(-0.55, 0.68, 0.48).normalize()
}

pub struct RenduPlugin;

impl Plugin for RenduPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(COULEUR_HORIZON))
            .insert_resource(GlobalAmbientLight {
                // Lumière diffuse du ciel, à peine chaude : les faces à l'ombre restent lisibles
                color: Color::srgb(1.0, 0.97, 0.92),
                brightness: 560.0,
                ..default()
            })
            .add_systems(Startup, creer_decor)
            .add_systems(Update, (animer_mer, centrer_ciel));
    }
}

#[derive(Component)]
struct Mer;

#[derive(Component)]
struct Ciel;

/// Matière mate, sans reflet : l'aspect « éclairage par sommet » de l'époque.
pub fn matiere_mate(texture: Handle<Image>, repetitions: Vec2) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        uv_transform: Affine2::from_scale(repetitions),
        ..default()
    }
}

fn creer_decor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Soleil de midi. Pas d'ombres portées calculées : à l'époque, une simple ombre ronde sous le personnage.
    commands.spawn((
        DirectionalLight {
            // Soleil de midi, légèrement doré
            color: Color::srgb(1.0, 0.96, 0.86),
            illuminance: 3_200.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(direction_soleil() * 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Mer : un grand disque transparent dont la couleur passe du turquoise clair près de la plage
    // au bleu profond vers l'horizon. On voit le fond à travers près de l'île.
    commands.spawn((
        Mer,
        Mesh3d(meshes.add(maillage_mer())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(images.add(texture_eau())),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            fog_enabled: false,
            ..default()
        })),
        Transform::from_xyz(0.0, NIVEAU_MER, 0.0),
    ));

    // Fond du large, au-delà du terrain de l'île
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(3000.0, 3000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.30, 0.50),
            unlit: true,
            fog_enabled: false,
            ..default()
        })),
        Transform::from_xyz(0.0, -3.05, 0.0),
    ));

    // Ciel : une grande sphère vue de l'intérieur, peinte comme un ciel de Mario Sunshine
    // (bleu profond, gros cumulus sur l'horizon, voiles légers plus haut)
    let sphere = Sphere::new(450.0).mesh().uv(48, 24);
    let matiere_ciel = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(images.add(texture_ciel())),
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Ciel,
        Mesh3d(meshes.add(sphere)),
        MeshMaterial3d(matiere_ciel),
        // La sphère est générée avec ses pôles sur l'axe Z : on la redresse (zénith en haut)
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

fn animer_mer(
    time: Res<Time>,
    mers: Query<&MeshMaterial3d<StandardMaterial>, With<Mer>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for mer in &mers {
        if let Some(mut matiere) = materials.get_mut(&mer.0) {
            let t = time.elapsed_secs();
            let decalage = Vec2::new(
                (VITESSE_MER.x * t).fract(),
                (VITESSE_MER.y * t).fract() + 0.01 * (t * 0.7).sin(),
            );
            matiere.uv_transform = Affine2::from_translation(decalage);
        }
    }
}

/// Le ciel suit la caméra : on ne peut jamais s'en approcher.
fn centrer_ciel(
    camera: Single<&Transform, (With<Camera3d>, Without<Ciel>)>,
    mut ciels: Query<&mut Transform, With<Ciel>>,
) {
    for mut ciel in &mut ciels {
        ciel.translation = camera.translation;
    }
}

// ---------------------------------------------------------------------------
// Textures fabriquées par le code, volontairement en basse résolution.
// Elles seront remplacées par de vraies textures quand l'île sera modélisée.
// ---------------------------------------------------------------------------

/// Nombre pseudo-aléatoire entre 0 et 1, toujours le même pour un pixel donné.
pub fn hasard(x: u32, y: u32, graine: u32) -> f32 {
    let mut n = x.wrapping_mul(374_761_393)
        ^ y.wrapping_mul(668_265_263)
        ^ graine.wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    (n & 0xFFFF) as f32 / 65_535.0
}

fn image_depuis(taille: u32, pixel: impl Fn(u32, u32) -> [f32; 3]) -> Image {
    let mut donnees = Vec::with_capacity((taille * taille * 4) as usize);
    for y in 0..taille {
        for x in 0..taille {
            let [r, g, b] = pixel(x, y);
            donnees.extend_from_slice(&[
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: taille,
            height: taille,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        donnees,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // Répétition + filtrage bilinéaire : le léger flou typique des consoles de l'époque
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

#[allow(dead_code)]
pub fn texture_sable() -> Image {
    image_depuis(32, |x, y| {
        let grain = hasard(x, y, 1) * 0.10 - 0.05;
        let tache = if hasard(x / 4, y / 4, 2) > 0.8 {
            -0.04
        } else {
            0.0
        };
        [
            0.94 + grain + tache,
            0.83 + grain + tache,
            0.56 + grain * 0.8 + tache,
        ]
    })
}

pub fn texture_pierre() -> Image {
    image_depuis(32, |x, y| {
        // Dalles de 16 pixels avec un joint sombre
        let joint = x % 16 == 0 || y % 16 == 0;
        let dalle = hasard(x / 16, y / 16, 3) * 0.10;
        let grain = hasard(x, y, 4) * 0.06;
        let v = if joint { 0.42 } else { 0.62 + dalle + grain };
        [v, v * 0.97, v * 0.92]
    })
}

pub fn texture_terre_cuite() -> Image {
    image_depuis(32, |x, y| {
        // Briques décalées une rangée sur deux
        let rangee = y / 8;
        let decale = if rangee % 2 == 0 { x } else { x + 8 };
        let joint = y % 8 == 0 || decale % 16 == 0;
        let brique = hasard(decale / 16, rangee, 5) * 0.12;
        let grain = hasard(x, y, 6) * 0.05;
        if joint {
            [0.80, 0.72, 0.60]
        } else {
            [
                0.82 + brique + grain,
                0.40 + brique * 0.5 + grain,
                0.24 + grain,
            ]
        }
    })
}

/// Grain neutre presque blanc : donne de la matière aux surfaces colorées par ailleurs.
pub fn texture_grain() -> Image {
    image_depuis(32, |x, y| {
        let v = 0.90 + 0.10 * hasard(x, y, 9);
        [v, v, v]
    })
}

/// Détail du sol : un grain gris clair à deux échelles, multiplié par les couleurs du terrain.
pub fn texture_sol() -> Image {
    image_depuis(64, |x, y| {
        let fin = hasard(x, y, 13);
        let moyen = hasard(x / 4, y / 4, 14);
        let v = 0.80 + 0.12 * fin + 0.08 * moyen;
        [v, v, v]
    })
}

#[allow(dead_code)]
pub fn texture_bois() -> Image {
    image_depuis(32, |x, y| {
        // Planches de 8 pixels de large
        let joint = x % 8 == 0;
        let planche = hasard(x / 8, 0, 11) * 0.10;
        let veine = hasard(x, y / 6, 12) * 0.06;
        if joint {
            [0.35, 0.24, 0.14]
        } else {
            [0.62 + planche + veine, 0.44 + planche + veine, 0.26 + veine]
        }
    })
}

#[allow(dead_code)]
pub fn texture_herbe() -> Image {
    image_depuis(32, |x, y| {
        let grain = hasard(x, y, 7) * 0.14 - 0.07;
        let touffe = if hasard(x / 2, y / 3, 8) > 0.75 {
            0.06
        } else {
            0.0
        };
        [
            0.26 + grain * 0.5,
            0.68 + grain + touffe,
            0.22 + grain * 0.4,
        ]
    })
}

fn texture_eau() -> Image {
    use std::f32::consts::TAU;
    let n = 64;
    image_depuis(n, |x, y| {
        let u = x as f32 / n as f32;
        let v = y as f32 / n as f32;
        // Fréquences entières : la texture se répète sans raccord visible
        let onde = (TAU * (u * 2.0 + (TAU * v).sin() * 0.25)).sin()
            + (TAU * (v * 3.0 + (TAU * u * 2.0).sin() * 0.20)).sin()
            + (TAU * (u + v) * 2.0).sin() * 0.6;
        // Fines lignes claires là où les ondes se croisent
        let ecume = (1.0 - (onde.abs() * 2.2)).clamp(0.0, 1.0);
        let fond = 0.5 + 0.5 * (TAU * (u * 1.0 - v * 2.0)).sin();
        // Presque blanche : la teinte vient du dégradé de la mer, la texture n'ajoute
        // que les reflets clairs des vagues
        let v = 0.80 + 0.05 * fond + 0.40 * ecume;
        [v, v, v.min(1.0)]
    })
}

// ---------------------------------------------------------------------------
// Mer et ciel
// ---------------------------------------------------------------------------

/// Disque de mer en anneaux concentriques, avec une couleur et une opacité par anneau.
fn maillage_mer() -> Mesh {
    use std::f32::consts::TAU;
    let rayons = [
        0.0, 20.0, 35.0, 42.0, 47.0, 52.0, 60.0, 70.0, 85.0, 100.0, 120.0, 150.0, 200.0, 260.0,
        400.0, 700.0, 1200.0, 2000.0,
    ];
    let secteurs = 96;
    let couleur = |r: f32| -> [f32; 4] {
        let mut i = 0;
        while i + 1 < DEGRADE_MER.len() - 1 && DEGRADE_MER[i + 1].0 < r {
            i += 1;
        }
        let (r0, c0, a0) = DEGRADE_MER[i];
        let (r1, c1, a1) = DEGRADE_MER[i + 1];
        let t = ((r - r0) / (r1 - r0)).clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        let c = Color::srgb(
            c0[0] + (c1[0] - c0[0]) * t,
            c0[1] + (c1[1] - c0[1]) * t,
            c0[2] + (c1[2] - c0[2]) * t,
        )
        .to_linear();
        [c.red, c.green, c.blue, a0 + (a1 - a0) * t]
    };

    let mut positions = Vec::new();
    let mut normales = Vec::new();
    let mut couleurs = Vec::new();
    let mut uvs = Vec::new();
    for r in rayons {
        for k in 0..=secteurs {
            let a = k as f32 / secteurs as f32 * TAU;
            let (x, z) = (a.cos() * r, a.sin() * r);
            positions.push([x, 0.0, z]);
            normales.push([0.0, 1.0, 0.0]);
            couleurs.push(couleur(r));
            uvs.push([x / METRES_PAR_MOTIF_MER, z / METRES_PAR_MOTIF_MER]);
        }
    }
    let ligne = (secteurs + 1) as u32;
    let mut indices = Vec::new();
    for anneau in 0..(rayons.len() as u32 - 1) {
        for k in 0..secteurs as u32 {
            let a = anneau * ligne + k;
            let b = a + 1;
            let c = a + ligne + 1;
            let d = a + ligne;
            // Faces tournées vers le haut
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normales)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, couleurs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

/// Bruit doux en 3D (valeurs au hasard sur une grille, interpolées), entre 0 et 1.
fn bruit3(p: Vec3, graine: u32) -> f32 {
    let base = p.floor();
    let f = p - base;
    let f = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let h = |dx: f32, dy: f32, dz: f32| {
        let x = (base.x + dx + 4096.0) as u32;
        let y = (base.y + dy + 4096.0) as u32;
        let z = (base.z + dz + 4096.0) as u32;
        hasard(x, y, z.wrapping_mul(7919).wrapping_add(graine))
    };
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(h(0., 0., 0.), h(1., 0., 0.), f.x);
    let x10 = lerp(h(0., 1., 0.), h(1., 1., 0.), f.x);
    let x01 = lerp(h(0., 0., 1.), h(1., 0., 1.), f.x);
    let x11 = lerp(h(0., 1., 1.), h(1., 1., 1.), f.x);
    lerp(lerp(x00, x10, f.y), lerp(x01, x11, f.y), f.z)
}

/// Somme de bruits de plus en plus fins : des formes « nuageuses ».
fn fbm(p: Vec3, graine: u32) -> f32 {
    fbm_n(p, graine, 5)
}

fn fbm_n(p: Vec3, graine: u32, octaves: u32) -> f32 {
    let (mut total, mut amplitude, mut frequence, mut somme) = (0.0, 0.5, 1.0, 0.0);
    for octave in 0..octaves {
        total += amplitude * bruit3(p * frequence, graine + octave * 101);
        somme += amplitude;
        amplitude *= 0.5;
        frequence *= 2.03;
    }
    total / somme
}

fn lisse(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Texture du ciel (projection équirectangulaire : horizontalement le tour complet,
/// verticalement du zénith en haut au nadir en bas). Calculée une fois au démarrage.
fn texture_ciel() -> Image {
    use std::f32::consts::{FRAC_PI_2, TAU};
    let (largeur, hauteur) = (1024_u32, 512_u32);
    let zenith = Vec3::new(0.10, 0.42, 0.86);
    let milieu = Vec3::new(0.30, 0.62, 0.95);
    let horizon = Vec3::new(0.78, 0.90, 0.97);
    let blanc = Vec3::new(1.0, 1.0, 1.0);
    let ombre_nuage = Vec3::new(0.68, 0.77, 0.90);

    let mut donnees = Vec::with_capacity((largeur * hauteur * 4) as usize);
    for y in 0..hauteur {
        let v = (y as f32 + 0.5) / hauteur as f32;
        // Élévation : 1 au zénith, 0 à l'horizon, négatif en dessous
        let e = 1.0 - 2.0 * v;
        for x in 0..largeur {
            let couleur = if e <= 0.0 {
                horizon
            } else {
                let u = (x as f32 + 0.5) / largeur as f32;
                let a = u * TAU;
                // Coordonnées sur un cylindre : la texture se raccorde sans couture
                let (cx, cz) = (a.cos(), a.sin());
                let angle = e * FRAC_PI_2;

                // Dégradé : bleu profond en haut, très clair à l'horizon
                let t = e.powf(0.55);
                let mut c = if t < 0.5 {
                    horizon.lerp(milieu, t / 0.5)
                } else {
                    milieu.lerp(zenith, (t - 0.5) / 0.5)
                };

                // Voiles légers et étirés, plus haut dans le ciel
                let cirrus = fbm(Vec3::new(cx * 2.2, angle * 9.0, cz * 2.2), 7);
                let zone_cirrus = lisse(0.10, 0.30, e) * (1.0 - lisse(0.75, 0.95, e));
                let a_cirrus = lisse(0.55, 0.78, cirrus) * 0.55 * zone_cirrus;
                c = c.lerp(blanc, a_cirrus);

                // Gros cumulus posés sur l'horizon, séparés par des trouées de ciel bleu,
                // blancs en haut et bleutés en bas
                let p = Vec3::new(cx * 3.0, angle * 6.0, cz * 3.0);
                let presence = lisse(0.38, 0.60, fbm_n(Vec3::new(cx * 1.1, 0.0, cz * 1.1), 11, 3));
                let altitude = e / 0.35;
                let densite = fbm(p, 3) + 0.30 * presence - 0.42 * altitude;
                let a_cumulus = lisse(0.52, 0.60, densite) * lisse(0.0, 0.02, e);
                let haut = fbm_n(p, 3, 3) - fbm_n(p + Vec3::Y * 0.25, 3, 3);
                let lumiere = (haut * 5.0 + 0.55 + 0.6 * altitude).clamp(0.0, 1.0);
                let teinte = ombre_nuage.lerp(blanc, lumiere);
                c.lerp(teinte, a_cumulus)
            };
            donnees.extend_from_slice(&[
                (couleur.x.clamp(0.0, 1.0) * 255.0) as u8,
                (couleur.y.clamp(0.0, 1.0) * 255.0) as u8,
                (couleur.z.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: largeur,
            height: hauteur,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        donnees,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    });
    image
}
