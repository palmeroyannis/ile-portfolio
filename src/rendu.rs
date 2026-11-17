//! Rendu « GameCube » : ciel en dégradé, mer qui défile, textures basse résolution,
//! éclairage simple sans reflets.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// --- Réglages ---
pub const COULEUR_HORIZON: Color = Color::srgb(0.72, 0.88, 0.98);
const COULEUR_ZENITH: [f32; 3] = [0.10, 0.36, 0.86];
const COULEUR_HORIZON_RGB: [f32; 3] = [0.72, 0.88, 0.98];
pub const NIVEAU_MER: f32 = -0.35;
const TAILLE_MER: f32 = 900.0;
const REPETITIONS_MER: f32 = 150.0; // nombre de répétitions de la texture d'eau
const OPACITE_MER: f32 = 0.62; // 0 = invisible, 1 = opaque
const VITESSE_MER: Vec2 = Vec2::new(0.035, 0.02); // défilement, en répétitions par seconde

pub struct RenduPlugin;

impl Plugin for RenduPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(COULEUR_HORIZON))
            .insert_resource(GlobalAmbientLight {
                color: Color::srgb(0.85, 0.92, 1.0),
                brightness: 450.0,
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
            illuminance: 1_900.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(3.0, 10.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Mer : un plan transparent avec une texture qui défile. On voit le fond à travers :
    // turquoise clair au-dessus du sable, bleu profond au large.
    commands.spawn((
        Mer,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(TAILLE_MER, TAILLE_MER))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, OPACITE_MER),
            base_color_texture: Some(images.add(texture_eau())),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            uv_transform: Affine2::from_scale(Vec2::splat(REPETITIONS_MER)),
            ..default()
        })),
        Transform::from_xyz(0.0, NIVEAU_MER, 0.0),
    ));

    // Fond du large, au-delà du terrain de l'île
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(TAILLE_MER, TAILLE_MER))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.38, 0.50),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -3.05, 0.0),
    ));

    // Ciel : une grande sphère vue de l'intérieur, dégradé du zénith à l'horizon
    let mut sphere = Sphere::new(450.0).mesh().uv(24, 12);
    let couleurs: Vec<[f32; 4]> = match sphere.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => positions
            .iter()
            .map(|p| {
                // 0 à l'horizon (et dessous), 1 au zénith
                let t = (p[1] / 450.0).clamp(0.0, 1.0).powf(0.6);
                let melange = |k: usize| {
                    COULEUR_HORIZON_RGB[k] + (COULEUR_ZENITH[k] - COULEUR_HORIZON_RGB[k]) * t
                };
                // Les couleurs par sommet sont attendues en valeurs « linéaires »
                let c = Color::srgb(melange(0), melange(1), melange(2)).to_linear();
                [c.red, c.green, c.blue, 1.0]
            })
            .collect(),
        _ => Vec::new(),
    };
    if !couleurs.is_empty() {
        sphere.insert_attribute(Mesh::ATTRIBUTE_COLOR, couleurs);
    }
    let matiere_ciel = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Ciel,
        Mesh3d(meshes.add(sphere)),
        MeshMaterial3d(matiere_ciel),
        Transform::default(),
    ));

    // Nuages : des blocs blancs à facettes, loin dans le ciel
    let nuage = meshes.add(Sphere::new(1.0).mesh().ico(1).unwrap());
    let blanc = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        fog_enabled: false,
        ..default()
    });
    let nuages = [
        (Vec3::new(-180.0, 70.0, -260.0), Vec3::new(55.0, 16.0, 30.0)),
        (Vec3::new(-140.0, 78.0, -250.0), Vec3::new(35.0, 20.0, 26.0)),
        (Vec3::new(120.0, 90.0, -300.0), Vec3::new(70.0, 20.0, 34.0)),
        (Vec3::new(170.0, 84.0, -280.0), Vec3::new(40.0, 14.0, 24.0)),
        (Vec3::new(300.0, 65.0, 60.0), Vec3::new(60.0, 18.0, 32.0)),
        (Vec3::new(-320.0, 75.0, 120.0), Vec3::new(65.0, 22.0, 36.0)),
        (Vec3::new(40.0, 95.0, 330.0), Vec3::new(80.0, 20.0, 40.0)),
        (Vec3::new(-60.0, 110.0, -340.0), Vec3::new(45.0, 14.0, 26.0)),
    ];
    for (position, taille) in nuages {
        commands.spawn((
            Mesh3d(nuage.clone()),
            MeshMaterial3d(blanc.clone()),
            Transform::from_translation(position).with_scale(taille),
        ));
    }
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
            matiere.uv_transform =
                Affine2::from_scale_angle_translation(Vec2::splat(REPETITIONS_MER), 0.0, decalage);
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
        [
            0.10 + 0.06 * fond + 0.55 * ecume,
            0.78 + 0.06 * fond + 0.20 * ecume,
            0.80 + 0.05 * fond + 0.18 * ecume,
        ]
    })
}
