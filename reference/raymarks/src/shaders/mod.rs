use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

/// A layer of abstraction over loading shader files directly from the `src/shaders` directory,
/// for ease of use.
pub enum Shader {
    Rasterization,
}

impl Shader {
    /// Get the name for the source file of the shader.
    pub fn source_file(&self) -> &str {
        match self {
            Self::Rasterization => "rasterization.wgsl",
        }
    }

    /// Load the source code of the shader.
    pub fn load_source(&self) -> String {
        let path = Self::shader_directory().join(self.source_file());
        read_to_string(path).unwrap()
    }

    /// Get the directory containing the shader source files.
    pub fn shader_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("shaders")
    }
}
