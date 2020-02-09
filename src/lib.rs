pub mod render;
pub mod util;
pub mod api;

// NOTE about design: To future self never have getters that get a mutable self reference or return a mutable reference to a value no matter what