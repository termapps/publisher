use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    Aarch64AppleDarwin,
    X86_64AppleDarwin,
    X86_64UnknownLinuxGnu,
    I686UnknownLinuxGnu,
    X86_64UnknownLinuxMusl,
    X86_64PcWindowsMsvc,
    I686PcWindowsMsvc,
    Source,
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Target::Aarch64AppleDarwin => write!(f, "aarch64-apple-darwin"),
            Target::X86_64AppleDarwin => write!(f, "x86_64-apple-darwin"),
            Target::X86_64UnknownLinuxGnu => write!(f, "x86_64-unknown-linux-gnu"),
            Target::I686UnknownLinuxGnu => write!(f, "i686-unknown-linux-gnu"),
            Target::X86_64UnknownLinuxMusl => write!(f, "x86_64-unknown-linux-musl"),
            Target::X86_64PcWindowsMsvc => write!(f, "x86_64-pc-windows-msvc"),
            Target::I686PcWindowsMsvc => write!(f, "i686-pc-windows-msvc"),
            Target::Source => write!(f, ""),
        }
    }
}
