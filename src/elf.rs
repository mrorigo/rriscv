use std::ptr;

use elfloader::*;

use crate::{memory::MemoryOperations, mmu::MMU};

pub struct Loader<'a> {
    pub vbase: u64,
    memory: &'a mut MMU,
}

impl<'a> Loader<'a> {
    pub fn create(vbase: u64, memory: &mut MMU) -> Loader {
        Loader { vbase, memory }
    }
}

impl ElfLoader for Loader<'_> {
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), ElfLoaderErr> {
        for header in load_headers {
            println!(
                "allocate base = {:#x} size = {:#x} flags = {}",
                header.virtual_addr(),
                header.mem_size(),
                header.flags()
            );
        }
        Ok(())
    }

    fn relocate(&mut self, entry: RelocationEntry) -> Result<(), ElfLoaderErr> {
        use elfloader::arch::x86_64::RelocationTypes::*;
        use RelocationType::x86_64;

        let addr: *mut u64 = (self.vbase + entry.offset) as *mut u64;

        match entry.rtype {
            x86_64(R_AMD64_RELATIVE) => {
                // This type requires addend to be present
                let addend = entry
                    .addend
                    .ok_or(ElfLoaderErr::UnsupportedRelocationEntry)?;

                // This is a relative relocation, add the offset (where we put our
                // binary in the vspace) to the addend and we're done.
                println!("R_RELATIVE *{:p} = {:#x}", addr, self.vbase + addend);
                Ok(())
            }
            _ => Ok((/* not implemented */)),
        }
    }

    fn load(&mut self, _flags: Flags, base: VAddr, region: &[u8]) -> Result<(), ElfLoaderErr> {
        let start = base;
        let end = base + region.len() as u64;
        println!(
            "load: base={:#x} region into = {:#x} -- {:#x}  vbase: {:#x}",
            base, start, end, self.vbase
        );
        for offs in 0..region.len() {
            self.memory.write_single(offs as u64 + start, region[offs]);
        }
        Ok(())
    }

    fn tls(
        &mut self,
        tdata_start: VAddr,
        _tdata_length: u64,
        total_size: u64,
        _align: u64,
    ) -> Result<(), ElfLoaderErr> {
        let tls_end = tdata_start + total_size;
        println!(
            "Initial TLS region is at = {:#x} -- {:#x}",
            tdata_start, tls_end
        );
        Ok(())
    }
}
