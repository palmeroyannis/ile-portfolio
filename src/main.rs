//! Portfolio jeu vidéo — une île à explorer, avec une fiche par projet.

mod camera;
mod fiche;
mod joueur;
mod monde;
mod personnage;
mod projets;
mod rendu;

use avian3d::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Dans le navigateur, ne pas chercher de fichiers « .meta » à côté des images :
                // le serveur répond par la page d'accueil, ce qui fait échouer le chargement.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Portfolio".into(),
                        // Dans le navigateur : dessine dans le <canvas> de index.html et suit sa taille
                        canvas: Some("#jeu".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::default())
        // Gravité un peu plus forte que la réalité : sauts plus nerveux, comme dans un jeu de plateforme
        .insert_resource(Gravity(Vec3::NEG_Y * 25.0))
        .add_plugins((
            rendu::RenduPlugin,
            monde::MondePlugin,
            joueur::JoueurPlugin,
            personnage::PersonnagePlugin,
            camera::CameraPlugin,
            fiche::FichePlugin,
        ))
        .run();
}
