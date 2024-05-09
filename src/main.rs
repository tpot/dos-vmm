use nix::{
    fcntl,
    sys::stat::Mode,
    fcntl::OFlag,
};

const KVM_DEVICE: &str = "/dev/kvm";

fn main() {

    // Open fd /dev/kvm
    let kvm_fd: i32 = match fcntl::open(KVM_DEVICE, OFlag::O_RDWR, Mode::empty()) {
        Ok(fd)     => { fd },
        Err(errno) => { eprintln!("Error opening {KVM_DEVICE}: {errno}");
                        std::process::exit(1); }
    };

    println!("fd = {kvm_fd}");
}
