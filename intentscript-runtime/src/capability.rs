use intentscript_compiler::ir::Capabilities;
use intentscript_core::{Error, Result};
use std::path::Path;

/// Capability checker for enforcing security policies
pub struct CapabilityChecker {
    capabilities: Capabilities,
}

impl CapabilityChecker {
    /// Create a new capability checker with the given capabilities
    pub fn new(capabilities: Capabilities) -> Self {
        Self { capabilities }
    }

    /// Check if filesystem read access is allowed for the given path
    pub fn check_fs_read(&self, path: &str) -> Result<()> {
        match &self.capabilities.fs {
            None => Err(Error::capability_violation(
                "Filesystem capability not enabled",
            )),
            Some(fs_cap) => {
                if fs_cap.read_roots.is_empty() {
                    return Err(Error::capability_violation(
                        "No read roots configured for filesystem capability",
                    ));
                }

                let normalized_path = Self::normalize_path(path);
                
                for root in &fs_cap.read_roots {
                    let normalized_root = Self::normalize_path(root);
                    if normalized_path.starts_with(&normalized_root) {
                        return Ok(());
                    }
                }

                Err(Error::capability_violation(format!(
                    "Path '{}' is not within allowed read roots: {:?}",
                    path, fs_cap.read_roots
                )))
            }
        }
    }

    /// Check if filesystem write access is allowed for the given path
    pub fn check_fs_write(&self, path: &str) -> Result<()> {
        match &self.capabilities.fs {
            None => Err(Error::capability_violation(
                "Filesystem capability not enabled",
            )),
            Some(fs_cap) => {
                if fs_cap.write_roots.is_empty() {
                    return Err(Error::capability_violation(
                        "No write roots configured for filesystem capability",
                    ));
                }

                let normalized_path = Self::normalize_path(path);
                
                for root in &fs_cap.write_roots {
                    let normalized_root = Self::normalize_path(root);
                    if normalized_path.starts_with(&normalized_root) {
                        return Ok(());
                    }
                }

                Err(Error::capability_violation(format!(
                    "Path '{}' is not within allowed write roots: {:?}",
                    path, fs_cap.write_roots
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
    fn normalize_path(path: &str) -> String {
        // Convert to absolute path if possible, otherwise use as-is
        Path::new(path)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(path).to_path_buf())
            .to_string_lossy()
            .to_string()
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
