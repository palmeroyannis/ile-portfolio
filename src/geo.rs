//! Petit outil pour fabriquer des maillages « à facettes » par le code :
//! chaque triangle a sa propre normale et sa propre couleur, comme les modèles de l'époque.
//! Tous les objets construits ainsi partagent une seule matière blanche (couleurs par sommet).

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use crate::rendu::hasard;

#[derive(Default)]
pub struct Constructeur {
    positions: Vec<[f32; 3]>,
    normales: Vec<[f32; 3]>,
    couleurs: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
}

/// Couleur légèrement assombrie ou éclaircie (facteur autour de 1).
pub fn nuance(couleur: Color, facteur: f32) -> Color {
    let c = couleur.to_srgba();
    Color::srgba(
        c.red * facteur,
        c.green * facteur,
        c.blue * facteur,
        c.alpha,
    )
}

impl Constructeur {
    pub fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, couleur: Color) {
        let n = (b - a).cross(c - a);
        if n.length_squared() < 1e-12 {
            return;
        }
        let n = n.normalize();
        let lin = couleur.to_linear();
        let col = [lin.red, lin.green, lin.blue, lin.alpha];
        let abs = n.abs();
        for p in [a, b, c] {
            // Coordonnées de texture projetées sur le plan le plus proche de la face
            let uv = if abs.y >= abs.x && abs.y >= abs.z {
                [p.x, p.z]
            } else if abs.x >= abs.z {
                [p.z, p.y]
            } else {
                [p.x, p.y]
            };
            self.positions.push(p.to_array());
            self.normales.push(n.to_array());
            self.couleurs.push(col);
            self.uvs.push([uv[0] * 0.5, uv[1] * 0.5]);
        }
    }

    /// Quadrilatère a-b-c-d, sommets dans le sens inverse des aiguilles d'une montre vu de face.
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, couleur: Color) {
        self.triangle(a, b, c, couleur);
        self.triangle(a, c, d, couleur);
    }

    /// Les deux faces (feuilles, brins d'herbe, palmes).
    pub fn triangle_double(&mut self, a: Vec3, b: Vec3, c: Vec3, couleur: Color) {
        self.triangle(a, b, c, couleur);
        self.triangle(a, c, b, nuance(couleur, 0.85));
    }

    pub fn quad_double(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, couleur: Color) {
        self.quad(a, b, c, d, couleur);
        self.quad(a, d, c, b, nuance(couleur, 0.85));
    }

    /// Pavé orienté.
    pub fn boite(&mut self, centre: Vec3, taille: Vec3, rotation: Quat, couleur: Color) {
        let h = taille / 2.0;
        let p = |x: f32, y: f32, z: f32| centre + rotation * Vec3::new(x * h.x, y * h.y, z * h.z);
        let (a, b, c, d) = (
            p(-1., -1., 1.),
            p(1., -1., 1.),
            p(1., 1., 1.),
            p(-1., 1., 1.),
        );
        let (e, f, g, k) = (
            p(-1., -1., -1.),
            p(1., -1., -1.),
            p(1., 1., -1.),
            p(-1., 1., -1.),
        );
        self.quad(a, b, c, d, couleur); // +Z
        self.quad(f, e, k, g, couleur); // -Z
        self.quad(b, f, g, c, couleur); // +X
        self.quad(e, a, d, k, couleur); // -X
        self.quad(d, c, g, k, nuance(couleur, 1.04)); // dessus
        self.quad(e, f, b, a, nuance(couleur, 0.9)); // dessous
    }

    /// Cylindre à facettes (tronc, poteau, tonneau) entre deux points.
    pub fn cylindre(
        &mut self,
        bas: Vec3,
        haut: Vec3,
        rayon_bas: f32,
        rayon_haut: f32,
        cotes: usize,
        couleur: Color,
        fermer: bool,
    ) {
        let axe = (haut - bas).normalize_or(Vec3::Y);
        let rotation = Quat::from_rotation_arc(Vec3::Y, axe);
        for i in 0..cotes {
            let a0 = i as f32 / cotes as f32 * TAU;
            let a1 = (i + 1) as f32 / cotes as f32 * TAU;
            let d0 = rotation * Vec3::new(a0.cos(), 0.0, a0.sin());
            let d1 = rotation * Vec3::new(a1.cos(), 0.0, a1.sin());
            let (p0, p1) = (bas + d0 * rayon_bas, bas + d1 * rayon_bas);
            let (q0, q1) = (haut + d0 * rayon_haut, haut + d1 * rayon_haut);
            self.quad(p0, q0, q1, p1, couleur);
            if fermer {
                self.triangle(haut, q1, q0, nuance(couleur, 1.05));
                self.triangle(bas, p0, p1, nuance(couleur, 0.9));
            }
        }
    }

    /// Octaèdre (fleur, noix de coco, petit fruit, salade stylisée).
    pub fn losange(&mut self, centre: Vec3, taille: Vec3, couleur: Color) {
        let x = Vec3::X * taille.x;
        let y = Vec3::Y * taille.y;
        let z = Vec3::Z * taille.z;
        let equateur = [centre + x, centre + z, centre - x, centre - z];
        for i in 0..4 {
            let a = equateur[i];
            let b = equateur[(i + 1) % 4];
            self.triangle(centre + y, b, a, nuance(couleur, 1.05));
            self.triangle(centre - y, a, b, nuance(couleur, 0.85));
        }
    }

    /// Rocher : icosaèdre déformé au hasard, puis étiré et tourné.
    /// Renvoie les sommets (pour fabriquer une collision qui épouse la forme).
    pub fn rocher(
        &mut self,
        centre: Vec3,
        taille: Vec3,
        rotation: Quat,
        graine: u32,
        couleur: Color,
    ) -> Vec<Vec3> {
        let ico = Sphere::new(1.0).mesh().ico(0).unwrap();
        let Some(VertexAttributeValues::Float32x3(positions)) =
            ico.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return Vec::new();
        };
        let points: Vec<Vec3> = positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let bosse = 0.75 + 0.5 * hasard(i as u32, graine, 77);
                centre + rotation * (Vec3::from_array(*p) * bosse * taille)
            })
            .collect();
        let indices: Vec<usize> = match ico.indices() {
            Some(Indices::U32(v)) => v.iter().map(|&i| i as usize).collect(),
            Some(Indices::U16(v)) => v.iter().map(|&i| i as usize).collect(),
            None => (0..points.len()).collect(),
        };
        for (k, t) in indices.chunks(3).enumerate() {
            let teinte = 0.85 + 0.25 * hasard(k as u32, graine, 78);
            self.triangle(
                points[t[0]],
                points[t[1]],
                points[t[2]],
                nuance(couleur, teinte),
            );
        }
        points
    }

    pub fn construire(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normales)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.couleurs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
    }
}

/// Matière commune aux objets construits par `Constructeur` : blanche, mate, avec un léger grain.
pub fn matiere_couleurs(grain: Handle<Image>, double_face: bool) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(grain),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        cull_mode: if double_face {
            None
        } else {
            Some(bevy::render::render_resource::Face::Back)
        },
        ..default()
    }
}
