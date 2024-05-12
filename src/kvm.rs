use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use nix::{
    fcntl,
    fcntl::OFlag,
    sys::stat::Mode,
    ioctl_write_int_bad,
    request_code_none,
};

use kvm_bindings::{
    KVMIO,
};

pub struct Kvm {
    fd: OwnedFd,
    vm_fd: Option<OwnedFd>,
    pub vcpu_fd: Option<OwnedFd>,
}

use std::{num::NonZeroUsize};

const KVM_DEVICE: &str = "/dev/kvm";

ioctl_write_int_bad!(kvm_create_vm, request_code_none!(KVMIO, 0x01));
ioctl_write_int_bad!(kvm_create_vcpu, request_code_none!(KVMIO, 0x41));
ioctl_write_int_bad!(kvm_get_vcpu_mmap_size, request_code_none!(KVMIO, 0x04));
ioctl_write_int_bad!(kvm_run, request_code_none!(KVMIO, 0x80));

impl Kvm {

    pub fn new() -> Result<Self, std::io::Error> {
        // Open /dev/kvm
        match fcntl::open(KVM_DEVICE, OFlag::O_RDWR, Mode::empty()) {
            Ok(fd) => unsafe {
                assert!(fd != -1);
                return Ok(Kvm{
                    fd: FromRawFd::from_raw_fd(fd),
                    vm_fd: None,
                    vcpu_fd: None,
                });
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                return Err(
                    std::io::Error::from_raw_os_error(errno as i32)
                );
            },
        };
    }

    pub fn create_vm(&mut self) -> Result<(), std::io::Error> {
        // Create VM fd
        match unsafe {
            kvm_create_vm(self.fd.as_raw_fd(), 0)
        } {
            Ok(fd) => unsafe {
                assert!(fd != -1);
                self.vm_fd = Some(FromRawFd::from_raw_fd(fd));
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                return Err(
                    std::io::Error::from_raw_os_error(errno as i32)
                );
            },
        };

        // Create vCPU fd
        match unsafe {
            let vm_fd = self.vm_fd.as_ref().unwrap();
            kvm_create_vcpu(vm_fd.as_raw_fd(), 0)
        } {
            Ok(fd) => unsafe {
                assert!(fd != -1);
                self.vcpu_fd = Some(FromRawFd::from_raw_fd(fd));
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                return Err(
                    std::io::Error::from_raw_os_error(errno as i32)
                );
            },
        }
        return Ok(());
    }

    // Find size of the shared `kvm_run` mapping
    pub fn mmap_size(&self) -> Result<NonZeroUsize, std::io::Error> {
        let mmap_size = match unsafe {
             kvm_get_vcpu_mmap_size(self.fd.as_raw_fd(), 0)
        } {
            Ok(result) => {
                NonZeroUsize::new(
                    result.try_into().expect("mmap_size too big for usize!"))
                .expect("mmap_size is zero")
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                return Err(
                    std::io::Error::from_raw_os_error(errno as i32)
                );
            },
        };
        return Ok(mmap_size);
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        match unsafe {
            let vcpu_fd = self.vcpu_fd.as_ref().unwrap();
            kvm_run(vcpu_fd.as_raw_fd(), 0)
        } {
            Ok(_) => {},
            Err(errno) => {
                assert!(errno as i32 != 0);
                return Err(
                    std::io::Error::from_raw_os_error(errno as i32)
                );
            },
        };
        return Ok(());
    }
}
