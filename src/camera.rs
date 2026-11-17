//! Caméra à la 3e personne derrière le personnage.
//! - Elle pivote toute seule quand on va à gauche ou à droite (flèches, Q/D, stick).
//! - On peut aussi la tourner et l'incliner en maintenant un clic de souris et en glissant,
//!   ou avec le stick droit d'une manette. La molette règle le recul.

use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;

use crate::joueur::Joueur;
use crate::rendu::COULEUR_HORIZON;

// --- Réglages ---
const DISTANCE: f32 = 7.0; // recul de départ
const DISTANCE_MIN: f32 = 3.5;
const DISTANCE_MAX: f32 = 13.0;
const TANGAGE: f32 = 0.35; // inclinaison de départ vers le bas (radians)
const TANGAGE_MIN: f32 = -0.45; // caméra sous le personnage : on peut lever les yeux vers le soleil
const TANGAGE_MAX: f32 = 1.25; // presque à la verticale
const SENSIBILITE_SOURIS: f32 = 0.005; // radians par pixel
const SENSIBILITE_MANETTE: f32 = 2.2; // radians par seconde
const HAUTEUR_VISEE: f32 = 0.6; // la caméra vise un peu au-dessus du centre du personnage
const SUIVI: f32 = 12.0; // plus grand = caméra plus « collée »
const VITESSE_PIVOT: f32 = 1.9; // radians par seconde quand on va à fond à gauche ou à droite
const MARGE_OBSTACLE: f32 = 0.4; // la caméra s'arrête à cette distance d'un mur ou du sol
const RETOUR: f32 = 3.0; // vitesse à laquelle la caméra reprend son recul après un obstacle

// Halo lumineux façon Mario Sunshine : seules les zones très claires « débordent »
const INTENSITE_HALO: f32 = 0.13;
const SEUIL_HALO: f32 = 0.72;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_camera)
            .add_systems(Update, (piloter_camera, placer_camera).chain());
    }
}

#[derive(Component)]
pub struct OrbiteCamera {
    /// Rotation gauche/droite autour du personnage (radians)
    pub lacet: f32,
    /// Inclinaison vers le bas (radians)
    tangage: f32,
    /// Recul voulu par le joueur (molette)
    distance: f32,
    /// Point visé, qui suit le personnage en douceur
    cible: Vec3,
    /// Recul réel, raccourci quand un obstacle se trouve entre la caméra et le personnage
    recul: f32,
}

fn creer_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Pas d'anticrénelage : les arêtes restent nettes et légèrement crénelées, comme à l'époque
        Msaa::Off,
        // Couleurs brutes et saturées, sans traitement « cinéma »
        Tonemapping::None,
        Bloom {
            intensity: INTENSITE_HALO,
            prefilter: BloomPrefilter {
                threshold: SEUIL_HALO,
                threshold_softness: 0.35,
            },
            composite_mode: BloomCompositeMode::Additive,
            ..Bloom::NATURAL
        },
        // L'horizon se fond dans la couleur du ciel
        DistanceFog {
            color: COULEUR_HORIZON,
            falloff: FogFalloff::Linear {
                start: 30.0,
                end: 260.0,
            },
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 10.0),
        OrbiteCamera {
            lacet: 0.0,
            tangage: TANGAGE,
            distance: DISTANCE,
            cible: crate::monde::depart(),
            recul: DISTANCE,
        },
    ));
}

fn piloter_camera(
    time: Res<Time>,
    souris: Res<ButtonInput<MouseButton>>,
    mouvement: Res<AccumulatedMouseMotion>,
    molette: Res<AccumulatedMouseScroll>,
    manettes: Query<&Gamepad>,
    fiche: Res<crate::fiche::Fiche>,
    joueur: Single<&Joueur>,
    mut orbite: Single<&mut OrbiteCamera>,
) {
    let dt = time.delta_secs();
    if fiche.ouverte.is_some() {
        return;
    }

    // Aller à gauche ou à droite fait pivoter la caméra, qui reste ainsi dans le dos du personnage
    orbite.lacet -= joueur.commande.x * VITESSE_PIVOT * dt;

    // Souris : maintenir un bouton et glisser
    if souris.any_pressed([MouseButton::Left, MouseButton::Right]) {
        orbite.lacet -= mouvement.delta.x * SENSIBILITE_SOURIS;
        orbite.tangage += mouvement.delta.y * SENSIBILITE_SOURIS;
    }

    // Molette : recul (les souris donnent des « crans », les pavés tactiles des pixels)
    let cran = match molette.unit {
        MouseScrollUnit::Line => molette.delta.y,
        MouseScrollUnit::Pixel => molette.delta.y / 60.0,
    };
    orbite.distance -= cran * 0.8;

    // Manette : stick droit
    for manette in &manettes {
        let stick = manette.right_stick();
        if stick.length() > 0.15 {
            orbite.lacet -= stick.x * SENSIBILITE_MANETTE * dt;
            orbite.tangage -= stick.y * SENSIBILITE_MANETTE * dt;
        }
    }

    orbite.lacet = orbite.lacet.rem_euclid(2.0 * PI);
    orbite.tangage = orbite.tangage.clamp(TANGAGE_MIN, TANGAGE_MAX);
    orbite.distance = orbite.distance.clamp(DISTANCE_MIN, DISTANCE_MAX);
}

fn placer_camera(
    time: Res<Time>,
    recherche: SpatialQuery,
    joueur: Single<(Entity, &Transform), (With<Joueur>, Without<OrbiteCamera>)>,
    camera: Single<(&mut Transform, &mut OrbiteCamera)>,
) {
    let (entite_joueur, position_joueur) = *joueur;
    let (mut transform, mut orbite) = camera.into_inner();
    let dt = time.delta_secs();

    let visee = position_joueur.translation + Vec3::Y * HAUTEUR_VISEE;
    let t = (SUIVI * dt).min(1.0);
    orbite.cible = orbite.cible.lerp(visee, t);

    let rotation = Quat::from_euler(EulerRot::YXZ, orbite.lacet, -orbite.tangage, 0.0);
    let arriere = rotation * Vec3::Z;

    // Obstacle entre le personnage et la caméra ? On rapproche la caméra aussitôt,
    // et elle reprend son recul en douceur une fois l'obstacle dépassé.
    let filtre = SpatialQueryFilter::from_excluded_entities([entite_joueur]);
    let mut recul_possible = orbite.distance;
    if let Ok(direction) = Dir3::new(arriere) {
        if let Some(contact) =
            recherche.cast_ray(orbite.cible, direction, orbite.distance, true, &filtre)
        {
            recul_possible = (contact.distance - MARGE_OBSTACLE).max(0.8);
        }
    }
    if recul_possible < orbite.recul {
        orbite.recul = recul_possible;
    } else {
        orbite.recul += (recul_possible - orbite.recul) * (RETOUR * dt).min(1.0);
    }

    transform.translation = orbite.cible + arriere * orbite.recul;
    transform.look_at(orbite.cible, Vec3::Y);
}
