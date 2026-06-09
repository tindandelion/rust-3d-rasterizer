//! Procedural mesh builders (**[`cube`]**, **[`sphere`]**, **[`dodecahedron`]**, **[`torus`]**).

mod cube;
mod dodecahedron;
mod sphere;
mod torus;

pub use cube::cube;
pub use dodecahedron::dodecahedron;
pub use sphere::sphere;
pub use torus::torus;
