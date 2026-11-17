//! Le soleil : un disque éblouissant dans le ciel et, quand on le regarde,
//! des reflets d'objectif en chapelet à l'écran, comme dans Mario Sunshine.

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::joueur::Joueur;
use crate::rendu::direction_soleil;

const DISTANCE_SOLEIL: f32 = 400.0; // à l'intérieur de la sphère du ciel (450 m)
const TAILLE_SOLEIL: f32 = 16.0;

/// Reflets : position le long de la ligne soleil → centre de l'écran (0 = soleil, 1 = centre,
/// 2 = à l'opposé), diamètre en pixels, couleur et opacité maximale.
const REFLETS: [(f32, f32, [f32; 3], f32); 7] = [
    (0.0, 420.0, [1.0, 0.97, 0.85], 0.35),
    (0.45, 46.0, [1.0, 0.90, 0.55], 0.20),
    (0.75, 22.0, [0.70, 1.0, 0.85], 0.25),
    (1.15, 70.0, [0.55, 0.80, 1.0], 0.12),
    (1.35, 18.0, [1.0, 0.75, 0.55], 0.28),
    (1.65, 110.0, [0.70, 0.60, 1.0], 0.10),
    (1.95, 36.0, [0.60, 1.0, 0.90], 0.18),
];

pub struct SoleilPlugin;

impl Plugin for SoleilPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_soleil).add_systems(
            PostUpdate,
            (placer_soleil, afficher_reflets).after(TransformSystems::Propagate),
        );
    }
}

#[derive(Component)]
struct Disque;

#[derive(Component)]
struct Reflet(usize);

#[derive(Component)]
struct Eblouissement;

/// Intensité actuelle des reflets (lissée, pour un fondu quand le soleil passe derrière un obstacle).
#[derive(Resource, Default)]
struct Eclat(f32);

/// Tache ronde qui s'estompe vers les bords. `dans_la_couleur` : le fondu est dans la couleur
/// (pour un mélange additif) ; sinon il est dans la transparence (pour l'interface).
fn tache_douce(dans_la_couleur: bool) -> Image {
    let n = 64_u32;
    let mut donnees = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / n as f32 * 2.0 - Vec2::ONE;
            let f = (1.0 - d.length()).clamp(0.0, 1.0);
            let f = f * f;
            let v = (f * 255.0) as u8;
            if dans_la_couleur {
                donnees.extend_from_slice(&[v, v, v, 255]);
            } else {
                donnees.extend_from_slice(&[255, 255, 255, v]);
            }
        }
    }
    Image::new(
        Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        donnees,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn creer_soleil(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let halo_additif = images.add(tache_douce(true));
    let halo_ecran = images.add(tache_douce(false));
    commands.init_resource::<Eclat>();

    // Disque très lumineux (au-delà du blanc : le halo de la caméra le fait rayonner)
    let disque = meshes.add(Circle::new(TAILLE_SOLEIL).mesh().resolution(16));
    commands
        .spawn((
            Disque,
            Mesh3d(disque.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::LinearRgba(LinearRgba::rgb(6.0, 5.4, 4.2)),
                unlit: true,
                fog_enabled: false,
                ..default()
            })),
            Transform::default(),
        ))
        .with_children(|soleil| {
            // Couronne autour du disque, qui s'estompe doucement
            soleil.spawn((
                Mesh3d(meshes.add(Rectangle::new(TAILLE_SOLEIL * 7.0, TAILLE_SOLEIL * 7.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::LinearRgba(LinearRgba::rgb(0.9, 0.85, 0.7)),
                    base_color_texture: Some(halo_additif),
                    alpha_mode: AlphaMode::Add,
                    unlit: true,
                    fog_enabled: false,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, -0.5),
            ));
        });

    // Reflets d'objectif : des disques semi-transparents par-dessus l'image
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            // Sous la fiche projet et l'invite
            GlobalZIndex(5),
            Pickable::IGNORE,
        ))
        .with_children(|ecran| {
            ecran.spawn((
                Eblouissement,
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.98, 0.9, 0.0)),
            ));
            for (i, (_, taille, _, _)) in REFLETS.iter().enumerate() {
                ecran.spawn((
                    Reflet(i),
                    ImageNode::new(halo_ecran.clone()).with_color(Color::NONE),
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(*taille),
                        height: px(*taille),
                        ..default()
                    },
                    Visibility::Hidden,
                ));
            }
        });
}

/// Le soleil suit la caméra (on ne peut jamais s'en approcher) et se tourne vers elle.
fn placer_soleil(
    camera: Single<&GlobalTransform, With<Camera3d>>,
    mut disques: Query<&mut Transform, With<Disque>>,
) {
    let position_camera = camera.translation();
    for mut transform in &mut disques {
        let position = position_camera + direction_soleil() * DISTANCE_SOLEIL;
        transform.translation = position;
        // Le disque est dessiné sur sa face +Z : on oriente -Z à l'opposé de la caméra
        transform.look_at(position + (position - position_camera), Vec3::Y);
    }
}

fn afficher_reflets(
    time: Res<Time>,
    recherche: SpatialQuery,
    mut eclat: ResMut<Eclat>,
    camera: Single<(&Camera, &GlobalTransform)>,
    joueur: Single<Entity, With<Joueur>>,
    mut reflets: Query<
        (&Reflet, &mut Node, &mut ImageNode, &mut Visibility),
        Without<Eblouissement>,
    >,
    mut eblouissement: Single<&mut BackgroundColor, With<Eblouissement>>,
) {
    let (camera, transform_camera) = *camera;
    let soleil = direction_soleil();
    let position_camera = transform_camera.translation();

    // Le soleil est-il à l'écran, et pas caché par le relief ou un objet ?
    let mut cible = 0.0;
    let mut ecran_soleil = Vec2::ZERO;
    let mut centre = Vec2::ZERO;
    if let (Some(taille), true) = (
        camera.logical_viewport_size(),
        transform_camera.forward().dot(soleil) > 0.0,
    ) {
        centre = taille / 2.0;
        if let Ok(p) =
            camera.world_to_viewport(transform_camera, position_camera + soleil * DISTANCE_SOLEIL)
        {
            ecran_soleil = p;
            let marge = taille * 0.1;
            let dans_ecran = p.x > -marge.x
                && p.y > -marge.y
                && p.x < taille.x + marge.x
                && p.y < taille.y + marge.y;
            let filtre = SpatialQueryFilter::from_excluded_entities([*joueur]);
            let cache = recherche
                .cast_ray(
                    position_camera,
                    Dir3::new_unchecked(soleil),
                    300.0,
                    true,
                    &filtre,
                )
                .is_some();
            if dans_ecran && !cache {
                // Plus fort quand le soleil est proche du centre de l'écran
                let ecart = (p - centre).length() / centre.length();
                cible = (1.0 - ecart).clamp(0.0, 1.0);
            }
        }
    }
    eclat.0 += (cible - eclat.0) * (8.0 * time.delta_secs()).min(1.0);
    let force = eclat.0;

    eblouissement.0 = Color::srgba(1.0, 0.98, 0.9, 0.22 * force * force);

    for (reflet, mut node, mut couleur, mut visibilite) in &mut reflets {
        if force < 0.01 {
            *visibilite = Visibility::Hidden;
            continue;
        }
        let (position, taille, teinte, opacite) = REFLETS[reflet.0];
        let p = ecran_soleil + (centre - ecran_soleil) * position;
        node.left = px(p.x - taille / 2.0);
        node.top = px(p.y - taille / 2.0);
        couleur.color = Color::srgba(teinte[0], teinte[1], teinte[2], opacite * force);
        *visibilite = Visibility::Inherited;
    }
}
