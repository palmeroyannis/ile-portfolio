//! Caméra à la 3e personne, fixée derrière le personnage.
//! Elle n'est pas pilotée par le joueur : elle pivote toute seule quand il tourne.

use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

use crate::joueur::Joueur;
use crate::rendu::COULEUR_HORIZON;

// --- Réglages ---
const DISTANCE: f32 = 7.0; // recul derrière le personnage
const TANGAGE: f32 = 0.35; // inclinaison vers le bas (radians)
const HAUTEUR_VISEE: f32 = 0.6; // la caméra vise un peu au-dessus du centre du personnage
const SUIVI: f32 = 12.0; // plus grand = caméra plus « collée »
const VITESSE_PIVOT: f32 = 1.9; // radians par seconde quand on va à fond à gauche ou à droite
const MARGE_OBSTACLE: f32 = 0.4; // la caméra s'arrête à cette distance d'un mur ou du sol
const RETOUR: f32 = 3.0; // vitesse à laquelle la caméra reprend son recul après un obstacle

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_camera)
            .add_systems(Update, (pivoter_camera, placer_camera).chain());
    }
}

#[derive(Component)]
pub struct OrbiteCamera {
    /// Rotation gauche/droite autour du personnage (radians)
    pub lacet: f32,
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
            cible: crate::monde::depart(),
            recul: DISTANCE,
        },
    ));
}

/// Aller à gauche ou à droite fait pivoter la caméra, qui reste ainsi dans le dos du personnage.
/// Avancer ou reculer tout droit ne la fait pas bouger.
fn pivoter_camera(time: Res<Time>, joueur: Single<&Joueur>, mut orbite: Single<&mut OrbiteCamera>) {
    orbite.lacet -= joueur.commande.x * VITESSE_PIVOT * time.delta_secs();
    orbite.lacet = orbite.lacet.rem_euclid(2.0 * PI);
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

    let rotation = Quat::from_euler(EulerRot::YXZ, orbite.lacet, -TANGAGE, 0.0);
    let arriere = rotation * Vec3::Z;

    // Obstacle entre le personnage et la caméra ? On rapproche la caméra aussitôt,
    // et elle reprend son recul en douceur une fois l'obstacle dépassé.
    let filtre = SpatialQueryFilter::from_excluded_entities([entite_joueur]);
    let mut recul_possible = DISTANCE;
    if let Ok(direction) = Dir3::new(arriere) {
        if let Some(contact) = recherche.cast_ray(orbite.cible, direction, DISTANCE, true, &filtre)
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
