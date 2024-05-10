use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use nix::{
    fcntl,
    sys::stat::Mode,
    fcntl::OFlag,
    ioctl_write_int_bad, request_code_none,
};

use kvm_bindings::{
    KVMIO,
};

const KVM_DEVICE: &str = "/dev/kvm";

ioctl_write_int_bad!(kvm_create_vm, request_code_none!(KVMIO, 0x01));
ioctl_write_int_bad!(kvm_create_vcpu, request_code_none!(KVMIO, 0x41));

fn main() {

    // Open /dev/kvm
    let kvm_fd: OwnedFd = match fcntl::open(KVM_DEVICE, OFlag::O_RDWR, Mode::empty()) {
        Ok(fd) => unsafe {
            assert!(fd != -1);
            FromRawFd::from_raw_fd(fd)
        },
        Err(errno) => {
            eprintln!("Error opening {KVM_DEVICE}: {errno}");
            std::process::exit(1);
        },
    };

    println!("kvm_fd = {0}", AsRawFd::as_raw_fd(&kvm_fd));

    // Create VM
    let vm_fd: OwnedFd = match unsafe { kvm_create_vm(kvm_fd.as_raw_fd(), 0) } {
        Ok(fd) => unsafe {
            assert!(fd != -1);
            FromRawFd::from_raw_fd(fd)
        },
        Err(errno) => {
            eprintln!("Error in kvm_create_vm: {errno}");
            std::process::exit(1);
        },
    };

    println!("vm_fd = {0}", AsRawFd::as_raw_fd(&vm_fd));

    // Create vCPU
    let vcpu_fd: OwnedFd = match unsafe { kvm_create_vcpu(vm_fd.as_raw_fd(), 0) } {
        Ok(fd) => unsafe {
            assert!(fd != -1);
            FromRawFd::from_raw_fd(fd)
        },
        Err(errno) => {
            eprintln!("Error in kvm_create_vm: {errno}");
            std::process::exit(1);
        },
    };

    println!("vcpu_fd = {0}", AsRawFd::as_raw_fd(&vcpu_fd));
}
