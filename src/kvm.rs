use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use nix::{
    fcntl,
    fcntl::OFlag,
    sys::stat::Mode,
    ioctl_read,
    ioctl_write_int_bad,
    ioctl_write_ptr,
    request_code_none,
};

use kvm_bindings::{
    kvm_regs,
    kvm_sregs,
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
ioctl_read!(kvm_get_regs, KVMIO, 0x81, kvm_regs);
ioctl_write_ptr!(kvm_set_regs, KVMIO, 0x82, kvm_regs);
ioctl_read!(kvm_get_sregs, KVMIO, 0x83, kvm_sregs);
ioctl_write_ptr!(kvm_set_sregs, KVMIO, 0x84, kvm_sregs);
ioctl_write_int_bad!(kvm_run, request_code_none!(KVMIO, 0x80));

impl Kvm {

    pub fn new() -> Result<Self, std::io::Error> {
        // Open /dev/kvm
        match fcntl::open(KVM_DEVICE, OFlag::O_RDWR, Mode::empty()) {
            Ok(fd) => unsafe {
                assert!(fd != -1);
                Ok(Kvm{
                    fd: FromRawFd::from_raw_fd(fd),
                    vm_fd: None,
                    vcpu_fd: None,
                })
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                Err(std::io::Error::from_raw_os_error(errno as i32))
            },
        }
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
                return Err(std::io::Error::from_raw_os_error(errno as i32));
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
                Ok(())
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                Err(std::io::Error::from_raw_os_error(errno as i32))
            },
        }
    }

    // Find size of the shared `kvm_run` mapping
    pub fn mmap_size(&self) -> Result<NonZeroUsize, std::io::Error> {
        match unsafe {
            kvm_get_vcpu_mmap_size(self.fd.as_raw_fd(), 0)
        } {
            Ok(result) => {
                Ok(
                    NonZeroUsize::new(
                        result.try_into().expect("mmap_size too big for usize!"))
                    .expect("mmap_size is zero"))
            },
            Err(errno) => {
                assert!(errno as i32 != 0);
                Err(std::io::Error::from_raw_os_error(errno as i32))
            },
        }
    }

    pub fn get_vcpu_sregs(&self) -> Result<kvm_sregs, std::io::Error> {
        let mut sregs = kvm_sregs::default();
        match unsafe {
            let vcpu_fd = self.vcpu_fd.as_ref().unwrap();
            kvm_get_sregs(vcpu_fd.as_raw_fd(), &mut sregs)
        } {
            Ok(_)      => Ok(sregs),
            Err(errno) => { Err(std::io::Error::from_raw_os_error(errno as i32)) },
        }
    }

    pub fn set_vcpu_sregs(&self, regs: *const kvm_sregs) -> Result<(), std::io::Error> {
        match unsafe {
            let vcpu_fd = self.vcpu_fd.as_ref().unwrap();
            kvm_set_sregs(vcpu_fd.as_raw_fd(), regs)
        } {
            Ok(_)      => Ok(()),
            Err(errno) => {
                assert!(errno as i32 != 0);
                Err(std::io::Error::from_raw_os_error(errno as i32))
            },
        }
    }

    pub fn get_vcpu_regs(&self) -> Result<kvm_regs, std::io::Error> {
        let mut regs = kvm_regs::default();
        match unsafe {
            let vcpu_fd = self.vcpu_fd.as_ref().unwrap();
            kvm_get_regs(vcpu_fd.as_raw_fd(), &mut regs)
        } {
            Ok(_)      => Ok(regs),
            Err(errno) => Err(std::io::Error::from_raw_os_error(errno as i32)),
        }
    }

    pub fn set_vcpu_regs(&self, regs: *const kvm_regs) -> Result<(), std::io::Error> {
        match unsafe {
            let vcpu_fd = self.vcpu_fd.as_ref().unwrap();
            kvm_set_regs(vcpu_fd.as_raw_fd(), regs)
        } {
            Ok(_)      => Ok(()),
            Err(errno) => Err(std::io::Error::from_raw_os_error(errno as i32)),
        }
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        match unsafe {
            let vcpu_fd = self.vcpu_fd.as_ref().unwrap();
            kvm_run(vcpu_fd.as_raw_fd(), 0)
        } {
            Ok(_)      => Ok(()),
            Err(errno) => {
                assert!(errno as i32 != 0);
                Err(std::io::Error::from_raw_os_error(errno as i32))
            },
        }
    }
}
