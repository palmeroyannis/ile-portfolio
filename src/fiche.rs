//! Interaction avec les projets : l'invite « Voir le projet » près d'un repère,
//! et la fiche sobre qui s'ouvre par-dessus le jeu.

use bevy::prelude::*;

use crate::joueur::Joueur;
use crate::monde::Repere;
use crate::projets::{Projet, PROJETS};

// --- Réglages ---
const DISTANCE_INTERACTION: f32 = 4.5; // mètres

const BLEU_NUIT: Color = Color::srgba(0.04, 0.09, 0.18, 0.94);
const VOILE: Color = Color::srgba(0.0, 0.02, 0.06, 0.60);
const BLANC: Color = Color::srgb(0.96, 0.97, 0.98);
const GRIS: Color = Color::srgb(0.68, 0.73, 0.80);
const TRAIT: Color = Color::srgba(1.0, 1.0, 1.0, 0.35);
const TURQUOISE: Color = Color::srgb(0.20, 0.82, 0.80);

pub struct FichePlugin;

impl Plugin for FichePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fiche>()
            .add_systems(Startup, charger_polices)
            .add_systems(
                Update,
                (gerer_fiche, afficher_fiche, afficher_invite).chain(),
            );
    }
}

/// État de la fiche : quel projet est ouvert, et quelle photo est affichée.
#[derive(Resource, Default)]
pub struct Fiche {
    pub ouverte: Option<usize>,
    photo: usize,
    /// Projet à portée du personnage (fiche fermée)
    a_portee: Option<usize>,
}

#[derive(Resource)]
struct Polices {
    normal: Handle<Font>,
    gras: Handle<Font>,
}

#[derive(Component)]
struct RacineFiche;

#[derive(Component)]
struct RacineInvite;

fn charger_polices(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Polices {
        normal: assets.load("polices/Inter-Regular.otf"),
        gras: assets.load("polices/Inter-SemiBold.otf"),
    });
}

fn gerer_fiche(
    clavier: Res<ButtonInput<KeyCode>>,
    manettes: Query<&Gamepad>,
    joueur: Single<&Transform, With<Joueur>>,
    reperes: Query<(&Transform, &Repere)>,
    mut fiche: ResMut<Fiche>,
) {
    let mut action = clavier.just_pressed(KeyCode::KeyE) || clavier.just_pressed(KeyCode::Enter);
    let mut fermer = clavier.just_pressed(KeyCode::Escape);
    let mut suivante = clavier.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let mut precedente = clavier.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    for manette in &manettes {
        action |= manette.just_pressed(GamepadButton::West);
        fermer |= manette.just_pressed(GamepadButton::East);
        suivante |= manette.just_pressed(GamepadButton::DPadRight);
        precedente |= manette.just_pressed(GamepadButton::DPadLeft);
    }

    match fiche.ouverte {
        Some(numero) => {
            if action || fermer {
                fiche.ouverte = None;
                return;
            }
            let nombre = PROJETS[numero].photos.len();
            if nombre > 1 {
                if suivante {
                    fiche.photo = (fiche.photo + 1) % nombre;
                }
                if precedente {
                    fiche.photo = (fiche.photo + nombre - 1) % nombre;
                }
            }
        }
        None => {
            // Repère le plus proche, à l'horizontale
            let ici = joueur.translation.xz();
            let proche = reperes
                .iter()
                .map(|(transform, repere)| {
                    (transform.translation.xz().distance(ici), repere.projet)
                })
                .filter(|(distance, _)| *distance < DISTANCE_INTERACTION)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, projet)| projet);
            if fiche.a_portee != proche {
                fiche.a_portee = proche;
            }
            if let (true, Some(numero)) = (action, proche) {
                fiche.ouverte = Some(numero);
                fiche.photo = 0;
            }
        }
    }
}

fn police(source: &Handle<Font>, taille: f32) -> TextFont {
    TextFont {
        font: source.clone().into(),
        font_size: FontSize::Px(taille),
        ..default()
    }
}

/// Reconstruit la fiche à chaque changement (ouverture, fermeture, changement de photo).
fn afficher_fiche(
    mut commands: Commands,
    fiche: Res<Fiche>,
    polices: Res<Polices>,
    assets: Res<AssetServer>,
    anciennes: Query<Entity, With<RacineFiche>>,
) {
    if !fiche.is_changed() {
        return;
    }
    for entite in &anciennes {
        commands.entity(entite).despawn();
    }
    let Some(numero) = fiche.ouverte else {
        return;
    };
    let projet: &Projet = &PROJETS[numero];
    let photo = fiche.photo.min(projet.photos.len().saturating_sub(1));

    commands
        .spawn((
            RacineFiche,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(VOILE),
            GlobalZIndex(10),
        ))
        .with_children(|racine| {
            racine
                .spawn((
                    Node {
                        width: percent(84),
                        max_width: px(1200),
                        height: percent(82),
                        flex_direction: FlexDirection::Row,
                        column_gap: px(28),
                        padding: UiRect::all(px(28)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(BLEU_NUIT),
                    BorderColor::all(TRAIT),
                ))
                .with_children(|panneau| {
                    // --- Colonne de gauche : photos ---
                    panneau
                        .spawn(Node {
                            width: percent(56),
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(14),
                            ..default()
                        })
                        .with_children(|gauche| {
                            gauche
                                .spawn((
                                    Node {
                                        width: percent(100),
                                        flex_grow: 1.0,
                                        min_height: px(0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        overflow: Overflow::clip(),
                                        border: UiRect::all(px(1)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                                    BorderColor::all(TRAIT),
                                ))
                                .with_children(|cadre| {
                                    if let Some(chemin) = projet.photos.get(photo) {
                                        cadre.spawn((
                                            ImageNode::new(assets.load(*chemin)),
                                            Node {
                                                max_width: percent(100),
                                                max_height: percent(100),
                                                ..default()
                                            },
                                        ));
                                    }
                                });

                            if projet.photos.len() > 1 {
                                gauche
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: px(10),
                                        ..default()
                                    })
                                    .with_children(|vignettes| {
                                        for (i, chemin) in projet.photos.iter().enumerate() {
                                            vignettes.spawn((
                                                ImageNode::new(assets.load(*chemin))
                                                    .with_mode(NodeImageMode::Stretch),
                                                Node {
                                                    width: px(108),
                                                    height: px(72),
                                                    border: UiRect::all(px(2)),
                                                    ..default()
                                                },
                                                BorderColor::all(if i == photo {
                                                    TURQUOISE
                                                } else {
                                                    TRAIT
                                                }),
                                            ));
                                        }
                                    });
                            }
                        });

                    // --- Colonne de droite : texte ---
                    panneau
                        .spawn(Node {
                            flex_grow: 1.0,
                            flex_basis: px(0),
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                            overflow: Overflow::clip(),
                            ..default()
                        })
                        .with_children(|droite| {
                            droite.spawn((
                                Text::new(projet.titre),
                                police(&polices.gras, 32.0),
                                TextColor(BLANC),
                            ));
                            droite.spawn((
                                Text::new(projet.sous_titre),
                                police(&polices.normal, 15.0),
                                TextColor(TURQUOISE),
                            ));
                            trait_horizontal(droite);

                            for paragraphe in projet.paragraphes {
                                droite.spawn((
                                    Text::new(*paragraphe),
                                    police(&polices.normal, 16.0),
                                    TextColor(BLANC),
                                ));
                            }

                            if !projet.chiffres.is_empty() {
                                droite.spawn((
                                    Text::new("POINTS CLÉS"),
                                    police(&polices.gras, 13.0),
                                    TextColor(GRIS),
                                    Node {
                                        margin: UiRect::top(px(8)),
                                        ..default()
                                    },
                                ));
                                for (libelle, valeur) in projet.chiffres {
                                    droite
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceBetween,
                                            column_gap: px(16),
                                            ..default()
                                        })
                                        .with_children(|ligne| {
                                            ligne.spawn((
                                                Text::new(*libelle),
                                                police(&polices.normal, 15.0),
                                                TextColor(GRIS),
                                            ));
                                            ligne.spawn((
                                                Text::new(*valeur),
                                                police(&polices.gras, 15.0),
                                                TextColor(BLANC),
                                            ));
                                        });
                                }
                            }

                            // Pousse les commandes tout en bas
                            droite.spawn(Node {
                                flex_grow: 1.0,
                                ..default()
                            });
                            trait_horizontal(droite);
                            let aide = if projet.photos.len() > 1 {
                                "Q / D : photos        E ou Échap : fermer"
                            } else {
                                "E ou Échap : fermer"
                            };
                            droite.spawn((
                                Text::new(aide),
                                police(&polices.normal, 14.0),
                                TextColor(GRIS),
                            ));
                        });
                });
        });
}

fn trait_horizontal(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: percent(100),
            height: px(1),
            ..default()
        },
        BackgroundColor(TRAIT),
    ));
}

/// Petite invite en bas de l'écran quand un projet est à portée.
fn afficher_invite(
    mut commands: Commands,
    fiche: Res<Fiche>,
    polices: Res<Polices>,
    anciennes: Query<Entity, With<RacineInvite>>,
) {
    if !fiche.is_changed() {
        return;
    }
    for entite in &anciennes {
        commands.entity(entite).despawn();
    }
    let (None, Some(numero)) = (fiche.ouverte, fiche.a_portee) else {
        return;
    };

    commands
        .spawn((
            RacineInvite,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                bottom: px(48),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|racine| {
            racine
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(12),
                        padding: UiRect::axes(px(18), px(10)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(BLEU_NUIT),
                    BorderColor::all(TRAIT),
                ))
                .with_children(|boite| {
                    boite.spawn((
                        Text::new("E"),
                        police(&polices.gras, 16.0),
                        TextColor(TURQUOISE),
                    ));
                    boite.spawn((
                        Text::new(format!("Voir le projet : {}", PROJETS[numero].titre)),
                        police(&polices.normal, 16.0),
                        TextColor(BLANC),
                    ));
                });
        });
}
