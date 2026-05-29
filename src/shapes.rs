//! Procedural mesh builders (**[`cube`]**, **[`sphere`]**, **[`dodecahedron`]**).

mod cube;
mod dodecahedron;
mod flat_sphere;
mod sphere;

pub use cube::cube;
pub use dodecahedron::dodecahedron;
pub use flat_sphere::sphere as flat_sphere;
pub use sphere::sphere;
