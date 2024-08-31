use nix::libc::siginfo_t;
use std::error::Error;
use std::os::raw::{c_int, c_void};

mod runner;  

extern "C" fn sigsegv_handler(_signal: c_int, siginfo: *mut siginfo_t, _extra: *mut c_void) {
    let address = unsafe { (*siginfo).si_addr() } as usize;

}

fn exec(filename: &str) -> Result<(), Box<dyn Error>> {
    // Read and parse the ELF file
    let elf_file = runner::parse_elf_file(filename)?;

    
    for (i, segment) in elf_file.segments.iter().enumerate() {
        eprintln!(
            "{}\t0x{:x}\t{}\t0x{:x}\t{}\t{}",
            i,
            segment.address,
            segment.size,
            segment.offset,
            segment.length,
            segment.flags
        );
    }

    
    let base_address = elf_file.base_address;
    let entry_point = elf_file.entry_point;

    eprintln!("Base address 0x{:x}", base_address);
    eprintln!("Entry point 0x{:x}", entry_point);

  
    unsafe {
        let mut sa = std::mem::zeroed();
        sa.sa_sigaction = sigsegv_handler as usize;
        sa.sa_flags = nix::libc::SA_SIGINFO;
        nix::libc::sigaction(nix::libc::SIGSEGV, &sa, std::ptr::null_mut());
    }


    runner::exec_run(base_address, entry_point);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
   
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    // Execute elf file
    exec(&args[1])
}
