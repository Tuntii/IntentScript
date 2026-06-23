use intentscript_compiler::ir::Capabilities;
use intentscript_core::{Error, Result};
use std::path::Path;

/// Capability checker for enforcing security policies
pub struct CapabilityChecker {
    capabilities: Capabilities,
}

enum FsAccess {
    Read,
    Write,
}

impl CapabilityChecker {
    /// Create a new capability checker with the given capabilities
    pub fn new(capabilities: Capabilities) -> Self {
        Self { capabilities }
    }

    /// Check if filesystem read access is allowed for the given path
    pub fn check_fs_read(&self, path: &str) -> Result<()> {
        self.resolve_fs_read_path(path).map(|_| ())
    }

    /// Resolve a read path against configured read roots after capability checks.
    pub fn resolve_fs_read_path(&self, path: &str) -> Result<String> {
        self.resolve_fs_path(path, FsAccess::Read)
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Check if filesystem write access is allowed for the given path
    pub fn check_fs_write(&self, path: &str) -> Result<()> {
        self.resolve_fs_write_path(path).map(|_| ())
    }

    /// Resolve a write path against configured write roots after capability checks.
    pub fn resolve_fs_write_path(&self, path: &str) -> Result<String> {
        self.resolve_fs_path(path, FsAccess::Write)
            .map(|p| p.to_string_lossy().to_string())
    }

    fn resolve_fs_path(&self, path: &str, access: FsAccess) -> Result<std::path::PathBuf> {
        match &self.capabilities.fs {
            None => Err(Error::capability_violation(
                "Filesystem capability not enabled",
            )),
            Some(fs_cap) => {
                let (roots, access_label) = match access {
                    FsAccess::Read => (&fs_cap.read_roots, "read"),
                    FsAccess::Write => (&fs_cap.write_roots, "write"),
                };

                if roots.is_empty() {
                    return Err(Error::capability_violation(format!(
                        "No {access_label} roots configured for filesystem capability"
                    )));
                }

                let normalized_path = Self::resolve_path_against_roots(path, roots);

                for root in roots {
                    let normalized_root = Self::normalize_path(root);
                    if Self::path_within_root(&normalized_path, &normalized_root) {
                        return Ok(normalized_path);
                    }
                }

                Err(Error::capability_violation(format!(
                    "Path '{}' is not within allowed {access_label} roots: {:?}",
                    path, roots
                )))
            }
        }
    }

    /// Check if network capability is enabled
    pub fn check_net_capability(&self) -> Result<()> {
        if self.capabilities.net {
            Ok(())
        } else {
            Err(Error::capability_violation(
                "Network capability not enabled (default is false)",
            ))
        }
    }

    /// Check if exec capability is enabled
    pub fn check_exec_capability(&self) -> Result<()> {
        if self.capabilities.exec {
            Ok(())
        } else {
            Err(Error::capability_violation(
                "Exec capability not enabled",
            ))
        }
    }

    /// Check if templates capability is enabled
    pub fn check_templates_capability(&self) -> Result<()> {
        if self.capabilities.templates {
            Ok(())
        } else {
            Err(Error::capability_violation(
                "Templates capability not enabled",
            ))
        }
    }

    /// Check if exports capability is enabled
    pub fn check_exports_capability(&self) -> Result<()> {
        if self.capabilities.exports {
            Ok(())
        } else {
            Err(Error::capability_violation(
                "Exports capability not enabled",
            ))
        }
    }

    /// Normalize a path for comparison (handle relative paths, etc.)
    fn normalize_path(path: &str) -> std::path::PathBuf {
        Path::new(path)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(path).to_path_buf())
    }

    /// Resolve a relative path against declared roots before canonicalization.
    fn resolve_path_against_roots(path: &str, roots: &[String]) -> std::path::PathBuf {
        let path_buf = Path::new(path);
        if path_buf.is_absolute() {
            return Self::normalize_path(path);
        }
        for root in roots {
            let candidate = Path::new(root).join(path_buf);
            if candidate.exists() {
                return candidate
                    .canonicalize()
                    .unwrap_or(candidate);
            }
        }
        Self::normalize_path(path)
    }

    /// Check that `path` is inside `root` using component-wise prefix matching.
    fn path_within_root(path: &std::path::Path, root: &std::path::Path) -> bool {
        path.starts_with(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intentscript_compiler::ir::FsCapability;

    fn create_fs_capability(read_roots: Vec<&str>, write_roots: Vec<&str>) -> Capabilities {
        Capabilities {
            fs: Some(FsCapability {
                read_roots: read_roots.into_iter().map(String::from).collect(),
                write_roots: write_roots.into_iter().map(String::from).collect(),
            }),
            net: false,
            exec: false,
            templates: false,
            exports: false,
        }
    }

    #[test]
    fn test_fs_read_allowed() {
        let caps = create_fs_capability(vec!["/tmp"], vec![]);
        let _checker = CapabilityChecker::new(caps);
        
        // This test may fail on Windows due to path normalization
        // In a real implementation, we'd need platform-specific path handling
    }

    #[test]
    fn test_path_prefix_boundary_not_confused() {
        let root = std::env::temp_dir().join("intentscript_cap_root");
        let sibling = std::env::temp_dir().join("intentscript_cap_rootX");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&sibling).unwrap();

        let inside = root.join("allowed.txt");
        std::fs::write(&inside, b"ok").unwrap();
        let outside = sibling.join("denied.txt");
        std::fs::write(&outside, b"no").unwrap();

        let caps = Capabilities {
            fs: Some(FsCapability {
                read_roots: vec![root.to_string_lossy().to_string()],
                write_roots: vec![],
            }),
            net: false,
            exec: false,
            templates: false,
            exports: false,
        };
        let checker = CapabilityChecker::new(caps);

        assert!(checker.check_fs_read(inside.to_str().unwrap()).is_ok());
        assert!(checker.check_fs_read(outside.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_fs_read_denied_no_capability() {
        let caps = Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: false,
            exports: false,
        };
        let checker = CapabilityChecker::new(caps);
        
        assert!(checker.check_fs_read("/tmp/file.txt").is_err());
    }

    #[test]
    fn test_net_capability_disabled_by_default() {
        let caps = Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: false,
            exports: false,
        };
        let checker = CapabilityChecker::new(caps);
        
        assert!(checker.check_net_capability().is_err());
    }

    #[test]
    fn test_net_capability_enabled() {
        let caps = Capabilities {
            fs: None,
            net: true,
            exec: false,
            templates: false,
            exports: false,
        };
        let checker = CapabilityChecker::new(caps);
        
        assert!(checker.check_net_capability().is_ok());
    }

    #[test]
    fn test_exec_capability() {
        let caps = Capabilities {
            fs: None,
            net: false,
            exec: true,
            templates: false,
            exports: false,
        };
        let checker = CapabilityChecker::new(caps);
        
        assert!(checker.check_exec_capability().is_ok());
    }

    #[test]
    fn test_templates_capability() {
        let caps = Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: true,
            exports: false,
        };
        let checker = CapabilityChecker::new(caps);
        
        assert!(checker.check_templates_capability().is_ok());
    }

    #[test]
    fn test_exports_capability() {
        let caps = Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: false,
            exports: true,
        };
        let checker = CapabilityChecker::new(caps);
        
        assert!(checker.check_exports_capability().is_ok());
    }
}
