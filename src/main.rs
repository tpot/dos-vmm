use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::{num::NonZeroUsize};

use nix::{
    fcntl,
    sys::stat::Mode,
    fcntl::OFlag,
    ioctl_write_int_bad, request_code_none,
    sys::{mman, mman::MapFlags, mman::ProtFlags},
};

use kvm_bindings::{
    KVMIO,
    kvm_run,
};

const KVM_DEVICE: &str = "/dev/kvm";

ioctl_write_int_bad!(kvm_create_vm, request_code_none!(KVMIO, 0x01));
ioctl_write_int_bad!(kvm_create_vcpu, request_code_none!(KVMIO, 0x41));
ioctl_write_int_bad!(kvm_get_vcpu_mmap_size, request_code_none!(KVMIO, 0x04));
ioctl_write_int_bad!(kvm_run, request_code_none!(KVMIO, 0x80));

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

    // Find size of the shared `kvm_run` mapping
    let mmap_size = match unsafe {
            kvm_get_vcpu_mmap_size(kvm_fd.as_raw_fd(), 0)
        } {
            Ok(result) => {
                NonZeroUsize::new(
                    result.try_into()
                    .expect("mmap_size too big for usize!"))
                .expect("mmap_size is zero")
            },
            Err(errno) => {
                eprintln!("Error in kvm_get_vcpu_mmap_size: {errno}");
                std::process::exit(1);
            },
        };

    println!("mmap_size = {mmap_size}");

    // Create mmap of kvm_run struct
    let _kvm_run_map = match unsafe {
        mman::mmap(
            None,
            mmap_size,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            &vcpu_fd,
            0,
        )
    } {
        Ok(result) => result.as_ptr() as *mut kvm_run,
        Err(err) => {
            eprintln!("Error in mman::mmap(): {err}");
            std::process::exit(1);
        }
    };

    match unsafe {
        kvm_run(vcpu_fd.as_raw_fd(), 0)
    } {
        Ok(_) => {},
        Err(errno) => {
            eprintln!("Error in kvm_run: {errno}");
            std::process::exit(1);
        },
    };

    println!("Success!");
}
