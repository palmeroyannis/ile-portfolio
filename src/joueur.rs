//! Le personnage : déplacements, saut, clavier et manette.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::camera::OrbiteCamera;

// --- Réglages du ressenti (à ajuster en jouant) ---
const VITESSE: f32 = 6.0; // m/s
const ACCELERATION_SOL: f32 = 60.0; // m/s² : réactivité au sol
const ACCELERATION_AIR: f32 = 20.0; // m/s² : contrôle en l'air
const VITESSE_SAUT: f32 = 9.5; // m/s vers le haut au départ du saut
const TOLERANCE_SAUT: f32 = 0.12; // s : on peut encore sauter juste après avoir quitté un bord
const MEMOIRE_SAUT: f32 = 0.12; // s : un appui juste avant d'atterrir compte quand même
const VITESSE_ROTATION: f32 = 14.0;

const RAYON: f32 = 0.35;
const HAUTEUR_CYLINDRE: f32 = 1.0; // hauteur totale = 1.0 + 2 × 0.35 = 1.7 m

pub struct JoueurPlugin;

impl Plugin for JoueurPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, creer_joueur)
            .add_systems(Update, (deplacer_joueur, orienter_joueur, reapparaitre, placer_ombre).chain());
    }
}

#[derive(Component, Default)]
pub struct Joueur {
    /// Commande de déplacement en cours (x : gauche/droite, y : arrière/avant), lue par la caméra
    pub commande: Vec2,
    /// Vrai quand le personnage touche le sol (lu par l'animation)
    pub au_sol: bool,
    temps_depuis_sol: f32,
    temps_depuis_appui_saut: f32,
    direction_voulue: Vec3,
}

/// L'ombre ronde projetée sous le personnage, comme dans les jeux de l'époque.
#[derive(Component)]
struct Ombre;

const RAYON_OMBRE: f32 = 0.5;
const PORTEE_OMBRE: f32 = 12.0; // au-delà de cette hauteur, plus d'ombre

/// La partie visible du personnage (tourne vers la direction de marche).
#[derive(Component)]
pub struct Apparence;

fn creer_joueur(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let demi_hauteur = HAUTEUR_CYLINDRE / 2.0 + RAYON;

    commands.spawn((
        Ombre,
        Mesh3d(meshes.add(Circle::new(RAYON_OMBRE).mesh().resolution(12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.45),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::default(),
    ));

    commands
        .spawn((
            Joueur {
                commande: Vec2::ZERO,
                au_sol: false,
                temps_depuis_sol: 1.0,
                temps_depuis_appui_saut: 1.0,
                direction_voulue: Vec3::ZERO,
            },
            Transform::from_translation(crate::monde::depart()),
            Visibility::default(),
            RigidBody::Dynamic,
            Collider::capsule(RAYON, HAUTEUR_CYLINDRE),
            // Le personnage ne bascule jamais
            LockedAxes::ROTATION_LOCKED,
            // Pas de frottement : on pilote la vitesse nous-mêmes, et il ne « colle » pas aux murs
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            // Détection du sol : une petite sphère lancée vers le bas depuis le centre
            ShapeCaster::new(Collider::sphere(RAYON * 0.9), Vec3::ZERO, Quat::IDENTITY, Dir3::NEG_Y)
                .with_max_distance(demi_hauteur - RAYON * 0.9 + 0.15),
            // Mouvement fluide entre deux pas de calcul de la physique
            TransformInterpolation,
        ))
        .with_children(|parent| {
            crate::personnage::construire(parent, &mut meshes, &mut materials);
        });
}

fn deplacer_joueur(
    time: Res<Time>,
    clavier: Res<ButtonInput<KeyCode>>,
    manettes: Query<&Gamepad>,
    orbite: Single<&OrbiteCamera>,
    fiche: Res<crate::fiche::Fiche>,
    mut joueurs: Query<(&mut Joueur, &mut LinearVelocity, &mut Friction, &ShapeHits)>,
) {
    let dt = time.delta_secs();

    // --- Lecture des commandes ---
    // KeyW/A/S/D désignent la position physique des touches : ce sont ZQSD sur un clavier AZERTY.
    let mut commande = Vec2::ZERO;
    if clavier.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        commande.y += 1.0;
    }
    if clavier.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        commande.y -= 1.0;
    }
    if clavier.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        commande.x += 1.0;
    }
    if clavier.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        commande.x -= 1.0;
    }
    let mut saut_appuye = clavier.just_pressed(KeyCode::Space);

    for manette in &manettes {
        let stick = manette.left_stick();
        if stick.length() > 0.15 {
            commande += stick;
        }
        saut_appuye |= manette.just_pressed(GamepadButton::South);
    }
    commande = commande.clamp_length_max(1.0);

    // Fiche projet ouverte : le personnage ne bouge plus
    if fiche.ouverte.is_some() {
        commande = Vec2::ZERO;
        saut_appuye = false;
    }

    // --- Direction dans le monde, relative à la caméra ---
    let rotation_camera = Quat::from_rotation_y(orbite.lacet);
    let avant = rotation_camera * Vec3::NEG_Z;
    let droite = rotation_camera * Vec3::X;
    let direction = avant * commande.y + droite * commande.x;

    for (mut joueur, mut vitesse, mut frottement, contacts) in &mut joueurs {
        // Au sol si la sphère de détection touche une surface pas trop pentue
        let au_sol = contacts.iter().any(|contact| contact.normal1.y > 0.6);

        joueur.temps_depuis_sol = if au_sol { 0.0 } else { joueur.temps_depuis_sol + dt };
        joueur.temps_depuis_appui_saut = if saut_appuye {
            0.0
        } else {
            joueur.temps_depuis_appui_saut + dt
        };
        joueur.direction_voulue = direction;
        joueur.commande = commande;
        joueur.au_sol = au_sol;

        // À l'arrêt sur le sol, le personnage accroche : il ne glisse pas dans les pentes.
        // En mouvement, aucun frottement : il ne colle pas aux murs.
        let a_l_arret = au_sol && commande == Vec2::ZERO;
        let voulu = if a_l_arret {
            Friction::new(1.0).with_combine_rule(CoefficientCombine::Max)
        } else {
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min)
        };
        if *frottement != voulu {
            *frottement = voulu;
        }

        // Vitesse horizontale : on se rapproche de la vitesse voulue
        let cible = direction * VITESSE;
        let acceleration = if au_sol { ACCELERATION_SOL } else { ACCELERATION_AIR };
        let horizontale = Vec2::new(vitesse.x, vitesse.z)
            .move_towards(Vec2::new(cible.x, cible.z), acceleration * dt);
        vitesse.x = horizontale.x;
        vitesse.z = horizontale.y;

        // Saut
        if joueur.temps_depuis_appui_saut < MEMOIRE_SAUT && joueur.temps_depuis_sol < TOLERANCE_SAUT {
            vitesse.y = VITESSE_SAUT;
            joueur.temps_depuis_appui_saut = 1.0;
            joueur.temps_depuis_sol = 1.0;
        }
    }
}

fn orienter_joueur(
    time: Res<Time>,
    joueurs: Query<&Joueur>,
    mut apparences: Query<(&mut Transform, &ChildOf), With<Apparence>>,
) {
    for (mut transform, parent) in &mut apparences {
        let Ok(joueur) = joueurs.get(parent.parent()) else {
            continue;
        };
        if joueur.direction_voulue.length_squared() < 0.01 {
            continue;
        }
        let cible = Transform::default()
            .looking_to(joueur.direction_voulue, Vec3::Y)
            .rotation;
        let t = (VITESSE_ROTATION * time.delta_secs()).min(1.0);
        transform.rotation = transform.rotation.slerp(cible, t);
    }
}

/// Si le personnage s'avance trop loin dans l'eau, on le remet au point de départ.
fn reapparaitre(mut joueurs: Query<(&mut Position, &mut LinearVelocity), With<Joueur>>) {
    for (mut position, mut vitesse) in &mut joueurs {
        if position.y < -0.1 {
            position.0 = crate::monde::depart();
            vitesse.0 = Vec3::ZERO;
        }
    }
}

/// Pose l'ombre ronde sur la première surface trouvée sous le personnage.
fn placer_ombre(
    recherche: SpatialQuery,
    joueur: Single<(Entity, &Transform), (With<Joueur>, Without<Ombre>)>,
    ombre: Single<(&mut Transform, &mut Visibility), With<Ombre>>,
) {
    let (entite, position) = *joueur;
    let (mut transform, mut visibilite) = ombre.into_inner();

    let filtre = SpatialQueryFilter::from_excluded_entities([entite]);
    match recherche.cast_ray(position.translation, Dir3::NEG_Y, PORTEE_OMBRE, true, &filtre) {
        Some(contact) => {
            let point = position.translation + Vec3::NEG_Y * contact.distance;
            // Légèrement au-dessus de la surface pour éviter le scintillement
            transform.translation = point + contact.normal * 0.03;
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, contact.normal);
            // L'ombre rétrécit quand le personnage s'élève
            let hauteur = (contact.distance - (HAUTEUR_CYLINDRE / 2.0 + RAYON)).max(0.0);
            transform.scale = Vec3::splat((1.0 - hauteur / PORTEE_OMBRE * 2.0).clamp(0.35, 1.0));
            *visibilite = Visibility::Inherited;
        }
        None => *visibilite = Visibility::Hidden,
    }
}
