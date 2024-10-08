use nix::libc::{Elf32_Ehdr, Elf32_Phdr};
use std::arch::asm;
use std::fs::File;
use std::io::Read;
use std::env;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Elf32AuxV {
    pub a_type: u32,
    pub a_un: Elf32AuxVBindgenTy1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union Elf32AuxVBindgenTy1 {
    pub a_val: u32,
}

pub const AT_NULL: u32 = 0;
pub const AT_PHDR: u32 = 3;
pub const AT_BASE: u32 = 7;
pub const AT_ENTRY: u32 = 9;
pub const AT_EXECFN: u32 = 31;

extern "C" {
    static environ: *mut *mut u8;
}

pub struct Segment {
    pub address: usize,
    pub size: usize,
    pub offset: usize,
    pub length: usize,
    pub flags: String,
}

pub struct ElfFile {
    pub segments: Vec<Segment>,
    pub base_address: usize,
    pub entry_point: usize,
}

pub fn parse_elf_file(filename: &str) -> Result<ElfFile, Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let ehdr = unsafe { &*(buffer.as_ptr() as *const Elf32_Ehdr) };
    let mut segments = Vec::new();

    for i in 0..ehdr.e_phnum {
        let phdr = unsafe {
            &*(buffer.as_ptr().offset(ehdr.e_phoff as isize + (i * std::mem::size_of::<Elf32_Phdr>()) as isize) as *const Elf32_Phdr)
        };

        let flags = format!(
            "{}{}{}",
            if phdr.p_flags & 1 != 0 { 'x' } else { '-' },
            if phdr.p_flags & 2 != 0 { 'w' } else { '-' },
            if phdr.p_flags & 4 != 0 { 'r' } else { '-' }
        );

        segments.push(Segment {
            address: phdr.p_vaddr as usize,
            size: phdr.p_memsz as usize,
            offset: phdr.p_offset as usize,
            length: phdr.p_filesz as usize,
            flags,
        });
    }

    Ok(ElfFile {
        segments,
        base_address: segments[0].address,
        entry_point: ehdr.e_entry as usize,
    })
}

pub fn exec_run(base_address: usize, entry_point: usize) {
    // Access the ELF header
    let ehdr = unsafe { &*(base_address as *const u8 as *const Elf32_Ehdr) };
    
    // Access the program header
    let phdr = unsafe { 
        &*((base_address + (*ehdr).e_phoff as usize) as *const u8 as *const Elf32_Phdr) 
    };

    let mut auxv;

    // Access the environment variables and auxiliary vectors
    let env_address = unsafe {
        let mut env = environ;

        // Skip environment variables
        while !(*env).is_null() {
            env = env.offset(1);
        }
        env = env.offset(1);
        auxv = &mut *(env as *mut u8 as *mut Elf32AuxV);

      
        let argv = environ.offset(-(env::args().len() as isize + 2));

        *argv.offset(2) = *argv.offset(1);
        *argv.offset(1) = (env::args().len()-1) as *mut u8;

        argv.offset(1)
    };

   
    while auxv.a_type != AT_NULL {
        match auxv.a_type {
            AT_PHDR => auxv.a_un.a_val = phdr as *const Elf32_Phdr as u32,
            AT_BASE => auxv.a_un.a_val = 0,
            AT_ENTRY => auxv.a_un.a_val = ehdr.e_entry,
            AT_EXECFN => auxv.a_un.a_val = 0,
            _ => {}
        }
        auxv = unsafe { &mut *(auxv as *mut Elf32AuxV).offset(1) };
    }

    unsafe {
        asm!(
            "mov esp, ebx
            xor ebx, ebx
            xor ecx, ecx
            xor edx, edx
            xor ebp, ebp
            xor esi, esi
            xor edi, edi
            jmp eax",
            in("eax") entry_point, in("ebx") env_address);
    }
}
