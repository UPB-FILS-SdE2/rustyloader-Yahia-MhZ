use nix::libc::{sigaction, siginfo_t, SigAction, SigHandler, SigSet, SIGSEGV};
use std::error::Error;
use std::fs::File;
use std::os::raw::{c_int, c_void};
use std::os::unix::io::AsRawFd;
use std::slice;

mod runner;

extern "C" fn sigsegv_handler(_signal: c_int, siginfo: *mut siginfo_t, _extra: *mut c_void) {
    let address = unsafe { (*siginfo).si_addr() } as usize;

    eprintln!("Segmentation fault at address: 0x{:x}", address);
}

fn exec(filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    let mmap = unsafe {
        mmap::map(
            file.as_raw_fd(),
            mmap::AccessType::Read,
            0,
            mmap::map_len(file.metadata()?),
        )?
    };

    let ehdr = unsafe { &*(mmap.ptr as *const Elf32_Ehdr) };
    let phdr = unsafe {
        slice::from_raw_parts(
            mmap.ptr.offset(ehdr.e_phoff as isize) as *const Elf32_Phdr,
            ehdr.e_phnum as usize,
        )
    };

    for (i, ph) in phdr.iter().enumerate() {
        eprintln!(
            "{}\t0x{:x}\t{}\t0x{:x}\t{}\t{}",
            i,
            ph.p_vaddr,
            ph.p_filesz,
            ph.p_offset,
            ph.p_memsz,
            ph.p_flags
        );
    }

    let base_address = phdr
        .iter()
        .map(|ph| ph.p_vaddr)
        .min()
        .ok_or("Failed to determine Base address")?;

    let entry_point = ehdr.e_entry;

    eprintln!("Base address 0x{:x}", base_address);
    eprintln!("Entry point 0x{:x}", entry_point);

    unsafe {
        let sigaction = SigAction::new(
            SigHandler::SigAction(sigsegv_handler),
            SigSet::empty(),
            0,
        );
        sigaction::sigaction(SIGSEGV, &sigaction)?;
    }

    runner::exec_run(base_address as usize, entry_point as usize);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <ELF file>", args[0]);
        std::process::exit(1);
    }
    exec(&args[1])
}
