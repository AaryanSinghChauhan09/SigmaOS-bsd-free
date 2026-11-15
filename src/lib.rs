#![forbid(unsafe_code)]
#![deny(clippy::all)]

/// Configuration for a FreeBSD jail.
#[derive(Debug, Clone)]
pub struct JailConfig {
    pub jid: u32,
    pub name: String,
    pub hostname: String,
    pub ip4: Option<String>,
    pub allow_raw_sockets: bool,
}

/// A FreeBSD jail wrapping its configuration.
#[derive(Debug, Clone)]
pub struct Jail {
    pub config: JailConfig,
}

impl Jail {
    /// Create a new `Jail` from a `JailConfig`.
    pub fn new(config: JailConfig) -> Self {
        Self { config }
    }

    /// Return the filesystem path for this jail.
    pub fn jail_path(&self) -> String {
        format!("/jail/{}", self.config.name)
    }

    /// Return a devfs rule string for this jail.
    pub fn to_devfs_rule(&self) -> String {
        format!(
            "[devfsrules_jail_{}=10]\nadd include $devfsrules_hide_all\nadd path 'null' unhide\nadd path 'zero' unhide\nadd path 'random' unhide\nadd path 'urandom' unhide",
            self.config.name
        )
    }
}

/// A ZFS dataset with a name, mountpoint, and compression algorithm.
#[derive(Debug, Clone)]
pub struct ZfsDataset {
    pub name: String,
    pub mountpoint: String,
    pub compression: String,
}

impl ZfsDataset {
    /// Create a new `ZfsDataset`, defaulting compression to `"lz4"`.
    pub fn new(name: &str, mountpoint: &str) -> Self {
        Self {
            name: name.to_string(),
            mountpoint: mountpoint.to_string(),
            compression: "lz4".to_string(),
        }
    }

    /// Return the `zfs create` command string for this dataset.
    pub fn zfs_create_command(&self) -> String {
        format!(
            "zfs create -o mountpoint={} -o compression={} {}",
            self.mountpoint, self.compression, self.name
        )
    }
}

/// Configuration for a non-interactive `bsdinstall` run.
#[derive(Debug, Clone)]
pub struct BsdInstallConfig {
    pub target_disk: String,
    pub hostname: String,
    pub timezone: String,
}

impl BsdInstallConfig {
    /// Return a bsdinstall non-interactive script skeleton.
    pub fn bsdinstall_script(&self) -> String {
        format!(
            "#!/bin/sh\n# bsdinstall non-interactive script\nexport DISTRIBUTIONS=\"kernel.txz base.txz\"\nexport BSDINSTALL_DISTSITE=\"https://download.freebsd.org/ftp/releases/amd64/amd64/14.0-RELEASE/\"\n\n# Partitioning\ngpart destroy -F {disk} || true\ngpart create -s gpt {disk}\ngpart add -t freebsd-boot -s 512k {disk}\ngpart add -t freebsd-swap -s 2G {disk}\ngpart add -t freebsd-ufs {disk}\nnewfs /dev/{disk}p3\nmount /dev/{disk}p3 /mnt\n\n# Distribution install\nbsdinstall distfetch\nbsdinstall distextract\n\n# Configuration\nsysrc hostname=\"{hostname}\"\nbsdinstall time {timezone}\n\nbsdinstall config\nbsdinstall entropy",
            disk = self.target_disk,
            hostname = self.hostname,
            timezone = self.timezone,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jail() -> Jail {
        Jail::new(JailConfig {
            jid: 1,
            name: "web".to_string(),
            hostname: "web.local".to_string(),
            ip4: Some("10.0.0.1".to_string()),
            allow_raw_sockets: false,
        })
    }

    fn make_dataset() -> ZfsDataset {
        ZfsDataset::new("zpool/data", "/data")
    }

    // Arrange + Act + Assert: jail_path
    #[test]
    fn test_jail_path() {
        let jail = make_jail();
        let path = jail.jail_path();
        assert_eq!(path, "/jail/web");
    }

    // Arrange + Act + Assert: devfs_rule
    #[test]
    fn test_devfs_rule() {
        let jail = make_jail();
        let rule = jail.to_devfs_rule();
        assert!(rule.contains("devfsrules_jail_web"));
        assert!(rule.contains("add path 'null' unhide"));
    }

    // Arrange + Act + Assert: zfs_create_command
    #[test]
    fn test_zfs_create_command() {
        let ds = make_dataset();
        let cmd = ds.zfs_create_command();
        assert_eq!(
            cmd,
            "zfs create -o mountpoint=/data -o compression=lz4 zpool/data"
        );
    }

    // Arrange + Act + Assert: bsdinstall_script
    #[test]
    fn test_bsdinstall_script() {
        let cfg = BsdInstallConfig {
            target_disk: "ada0".to_string(),
            hostname: "myhost".to_string(),
            timezone: "UTC".to_string(),
        };
        let script = cfg.bsdinstall_script();
        assert!(script.contains("ada0"));
        assert!(script.contains("myhost"));
        assert!(script.contains("UTC"));
        assert!(script.starts_with("#!/bin/sh"));
    }

    // Arrange + Act + Assert: default compression
    #[test]
    fn test_default_compression() {
        let ds = ZfsDataset::new("pool/logs", "/logs");
        assert_eq!(ds.compression, "lz4");
    }

    // Arrange + Act + Assert: jail_new stores config
    #[test]
    fn test_jail_new() {
        let cfg = JailConfig {
            jid: 42,
            name: "db".to_string(),
            hostname: "db.local".to_string(),
            ip4: None,
            allow_raw_sockets: true,
        };
        let jail = Jail::new(cfg);
        assert_eq!(jail.config.jid, 42);
        assert_eq!(jail.config.name, "db");
        assert!(jail.config.allow_raw_sockets);
        assert!(jail.config.ip4.is_none());
    }

    // Arrange + Act + Assert: dataset_name preserved
    #[test]
    fn test_dataset_name() {
        let ds = ZfsDataset::new("tank/vms", "/vms");
        assert_eq!(ds.name, "tank/vms");
        assert_eq!(ds.mountpoint, "/vms");
    }
}
