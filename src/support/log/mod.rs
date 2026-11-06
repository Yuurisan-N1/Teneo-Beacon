pub mod format;
pub mod init;

pub use format::*;
pub use init::*;

pub fn install() {
    init::arm();
}
