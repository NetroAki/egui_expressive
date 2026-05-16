//! Material 3 core components.

#[path = "components/button.rs"]
mod button;
#[path = "components/feedback.rs"]
mod feedback;
#[path = "components/inputs.rs"]
mod inputs;
#[path = "components/surfaces.rs"]
mod surfaces;

pub use button::*;
pub use feedback::*;
pub use inputs::*;
pub use surfaces::*;
