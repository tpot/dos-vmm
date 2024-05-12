pub mod kvm;
use kvm::Kvm;

use kvm_bindings::{
    kvm_run,
};

use nix::{
   sys::{mman, mman::MapFlags, mman::ProtFlags},
};

fn main() {

    let mut k = Kvm::new().expect("Error opening /dev/kvm");
    k.create_vm().expect("Error creating VM: {}");

    let mmap_size = k.mmap_size().expect("mmap_size");

    // Create mmap of kvm_run struct
    let _kvm_run_map = match unsafe {
        let vcpu_fd = k.vcpu_fd.as_ref().unwrap();
        mman::mmap(
            None,
            mmap_size,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            vcpu_fd,
            0,
        )
    } {
        Ok(result) => result.as_ptr() as *mut kvm_run,
        Err(err) => {
            eprintln!("Error in mman::mmap(): {err}");
            std::process::exit(1);
        }
    };

    k.run().expect("kvm_run");

    println!("Success!");
}
