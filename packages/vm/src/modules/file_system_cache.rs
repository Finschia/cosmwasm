// copied from https://github.com/wasmerio/wasmer/blob/0.8.0/lib/runtime/src/cache.rs
// with some minor modifications

use memmap::Mmap;
use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::PathBuf,
};

use wasmer_runtime_core::{cache::Artifact, module::Module};

use crate::backends::{compiler_for_backend, BACKEND_NAME};
use crate::checksum::Checksum;
use crate::errors::{VmError, VmResult};

/// Representation of a directory that contains compiled Wasm artifacts.
pub struct FileSystemCache {
    path: PathBuf,
}

impl FileSystemCache {
    /// Construct a new `FileSystemCache` around the specified directory.
    /// The contents of the cache are stored in sub-versioned directories.
    ///
    /// # Safety
    ///
    /// This method is unsafe because there's no way to ensure the artifacts
    /// stored in this cache haven't been corrupted or tampered with.
    pub unsafe fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let path: PathBuf = path.into();
        if path.exists() {
            let metadata = path.metadata()?;
            if metadata.is_dir() {
                if !metadata.permissions().readonly() {
                    Ok(Self { path })
                } else {
                    // This directory is readonly.
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("the supplied path is readonly: {}", path.display()),
                    ))
                }
            } else {
                // This path points to a file.
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "the supplied path already points to a file: {}",
                        path.display()
                    ),
                ))
            }
        } else {
            // Create the directory and any parent directories if they don't yet exist.
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    pub fn load(&self, checksum: &Checksum) -> VmResult<Option<Module>> {
        let backend = BACKEND_NAME;

        let filename = checksum.to_hex();
        let file_path = self.path.clone().join(backend).join(filename);

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return Ok(None),
                _ => {
                    return Err(VmError::cache_err(format!(
                        "Error opening module file: {}",
                        err
                    )))
                }
            },
        };

        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| VmError::cache_err(format!("Mmap error: {}", e)))?;

        let serialized_cache = Artifact::deserialize(&mmap[..])?;
        let module = unsafe {
            wasmer_runtime_core::load_cache_with(
                serialized_cache,
                compiler_for_backend(backend)
                    .ok_or_else(|| VmError::cache_err(format!("Unsupported backend: {}", backend)))?
                    .as_ref(),
            )
        }?;
        Ok(Some(module))
    }

    /// Stores a serialization of the module to the file system
    pub fn store(&mut self, checksum: &Checksum, module: &Module) -> VmResult<()> {
        let backend_str = module.info().backend.to_string();
        let modules_dir = self.path.clone().join(backend_str);
        fs::create_dir_all(&modules_dir)
            .map_err(|e| VmError::cache_err(format!("Error creating direcory: {}", e)))?;

        let serialized_cache = module.cache()?;
        let buffer = serialized_cache.serialize()?;

        let filename = checksum.to_hex();
        let mut file = File::create(modules_dir.join(filename))
            .map_err(|e| VmError::cache_err(format!("Error creating module file: {}", e)))?;
        file.write_all(&buffer)
            .map_err(|e| VmError::cache_err(format!("Error writing module to disk: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::compile;
    use tempfile::TempDir;

    #[test]
    fn test_file_system_cache_run() {
        use wasmer_runtime_core::{imports, typed_func::Func};

        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path()).unwrap() };

        // Create module
        let wasm = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
            "#,
        )
        .unwrap();
        let checksum = Checksum::generate(&wasm);
        let module = compile(&wasm).unwrap();

        // Module does not exist
        let cached = cache.load(&checksum).unwrap();
        assert!(cached.is_none());

        // Store module
        cache.store(&checksum, &module).unwrap();

        // Load module
        let cached = cache.load(&checksum).unwrap();
        assert!(cached.is_some());

        // Check the returned module is functional.
        // This is not really testing the cache API but better safe than sorry.
        {
            assert_eq!(module.info().backend.to_string(), BACKEND_NAME.to_string());
            let cached_module = cached.unwrap();
            let import_object = imports! {};
            let instance = cached_module.instantiate(&import_object).unwrap();
            let add_one: Func<i32, i32> = instance.exports.get("add_one").unwrap();
            let value = add_one.call(42).unwrap();
            assert_eq!(value, 43);
        }
    }
}
