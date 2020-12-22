extern crate pmw1;

use std::env::args;
use std::io::prelude::*;
use std::fs::{File,OpenOptions};

use pmw1::exe::Pmw1Exe;

const OBJECTSFONCTIONS_LOC: u32 = 0x6580;
const TYPE_OUYE: u16 = 41; // The object type from which we're copying collision code.
const TYPE_GROSPIC: u16 = 107; // This has different collision code to TYPE_OUYE, although the functionality is identical. We'll exploit this!
const DEST_TYPES: [u16; 4] = [ // The object types to which we're copying collision code.
    45, // TYPE_MOVE_OUYE
    //101, // TYPE_MORNINGSTAR_MOUNTAI - not suitable, since a HitPoints value of 1 is used in stock KIT for the shootable one (albeit for no good reason...)
    105, // TYPE_MARTEAU
    106, // TYPE_MOVE_MARTEAU
    107, // TYPE_GROSPIC - yep, this too!
];
const GROSPIC_DEST_TYPES: [u16; 1] = [ // The object types to which we're copying alternative collision code
    101, // TYPE_MORNINGSTAR_MOUNTAI
];

// Find the location of a collision function pointer in RAYKIT.EXE's data section.
fn collision_fptr_location(object_type: u16) -> u32 {
    OBJECTSFONCTIONS_LOC + 20*(object_type as u32) + 8 // There are five function pointers per object type, and the collision function is the third one.
}

fn main() -> std::io::Result<()> {
    // Assume the filename of interest is the LAST argument on the command line.
    let exe_name: String = args().next_back().unwrap();

    // Load the whole EXE into memory...
    let binary = {
        println!("Opening {}...", exe_name);

        let mut file = File::open(&exe_name)?;
        let mut buffer: Vec<u8> = Vec::with_capacity(0x100000);
        file.read_to_end(&mut buffer)?;
        buffer.shrink_to_fit();
        buffer
    };

    // Create a backup file.
    {
        // IPR for "instapricklies", to distinguish from BAK file from the other patcher.
        let filename = format!("{}.BAK.IPR",exe_name);
        println!("");
        println!("Attempting to create NEW backup file {}", filename);
        // `create_new` to fail if the backup file already exists.
        // Don't wanna screw up an existing backup...
        let mut outfile = OpenOptions::new().write(true)
                                            .create_new(true)
                                            .open(&filename)?;
        // Write the whole binary back out
        outfile.write_all(&binary)?;
        println!("Backup successful");
    }

    println!("{} is {} bytes.", exe_name, binary.len());

    assert_eq!(binary[0..2],b"MZ"[..],
               "{} is not an MZ executable!", exe_name);
    assert!(binary.len() >= 0x1c,
            "{} doesn't appear to contain a complete MZ header!",exe_name);

    let mz_header = &binary[0x2..0x1c];
    let mz_header: Vec<u16> = (0..mz_header.len())
        .step_by(2)
        .map(|i| u16::from_le_bytes([mz_header[i], mz_header[i+1]]))
        .collect();

    // Print out some relevant info.
    println!("It begins with an MZ executable, of {} half-KiB blocks.",
             mz_header[1]);
    let total_block_size = mz_header[1] << 9; // Shift left to multiply by 512
    let actual_mz_size =
        if mz_header[0] == 0 {
            println!("Last block is fully used.");
            total_block_size
        } else {
            println!("{} bytes used in last block.", mz_header[0]);
            total_block_size - 512 + mz_header[0]
        } as usize;
    println!("Total MZ executable size is {} bytes.", actual_mz_size);

    assert!(binary.len() > actual_mz_size, "This appears to be a pure MZ executable!");

    // A slice containing just the PMW1 part.
    // Decompress the EXE immediately since we're going to be manipulating the relocation data.
    println!("");
    let mut pmw1_exe = Pmw1Exe::from_bytes(&binary[actual_mz_size..])?.decompress()?;

    // Get the data section, which contains the function-pointer table.
    let data_section = pmw1_exe.stack_object_mut();

    // First, find the address of the TYPE_OUYE collision function, by iterating over the
    // relocation entries.
    println!("Finding address of TYPE_OUYE collision function...");
    let ouye_collision_fptr = data_section
        .iter_reloc_blocks()
        .flat_map(|block| block.iter_reloc_entries().unwrap())
        .find(|entry| entry.source == collision_fptr_location(TYPE_OUYE))
        .expect(&format!("Relocation entry not found for TYPE_OUYE collision function - you may have the wrong (version of the) {} file!", exe_name))
        .target;
    assert_eq!(ouye_collision_fptr, 0x1BEF4,
            "Relocation entry for TYPE_OUYE collision function has unexpected target - you may have the wrong (version of the) {} file!", exe_name);

    println!("Finding address of TYPE_GROSPIC collision function...");
    let alt_collision_fptr = data_section
        .iter_reloc_blocks()
        .flat_map(|block| block.iter_reloc_entries().unwrap())
        .find(|entry| entry.source == collision_fptr_location(TYPE_GROSPIC))
        .expect(&format!("Relocation entry not found for TYPE_GROSPIC collision function - you may have the wrong (version of the) {} file!", exe_name))
        .target;
    assert_eq!(alt_collision_fptr, 0x1BEDC,
            "Relocation entry for TYPE_GROSPIC collision function has unexpected target - you may have the wrong (version of the) {} file!", exe_name);

    // Now, point all the other object types to the same collision function, so that they will
    // insta-kill Rayman if and only if their hitpoints value is 1.
    println!("Updating other collision function pointers...");
    let collision_fptr_locs: Vec<_> = DEST_TYPES
        .iter()
        .map(|&obj_type| collision_fptr_location(obj_type))
        .collect();
    for reloc_entry in data_section.iter_reloc_blocks_mut().flat_map(|block| block.iter_reloc_entries_mut().unwrap()).filter(|entry| collision_fptr_locs.contains(&entry.source)) {
        reloc_entry.target = ouye_collision_fptr;
    }

    println!("Updating alternative collision function pointers...");
    let collision_fptr_locs: Vec<_> = GROSPIC_DEST_TYPES
        .iter()
        .map(|&obj_type| collision_fptr_location(obj_type))
        .collect();
    for reloc_entry in data_section.iter_reloc_blocks_mut().flat_map(|block| block.iter_reloc_entries_mut().unwrap()).filter(|entry| collision_fptr_locs.contains(&entry.source)) {
        reloc_entry.target = alt_collision_fptr;
    }

    // Now change the code of the alternative collision function to look for HitPoints >= 2 instead
    // of == 1.
    println!("Updating alternative collision function code...");
    pmw1_exe.entry_object_mut().update_data(|data| {
        let jump_opptr = (alt_collision_fptr as usize) + 5;
        assert_eq!(data[jump_opptr], 0x75,
            "`jnz` opcode not found in expected location in TYPE_GROSPIC collision function - you may have the wrong (version of the) {} file!", exe_name);
        
        let mut new_data = data.to_vec();
        new_data[jump_opptr] = 0x76; // Change `jnz` to `jbe` so the condition for skipping is now HitPoints <= 1 instead of HitPoints != 1
        new_data
    })?;

    pmw1_exe = pmw1_exe.compress()?;
    println!("Done!");

    // Write out the patched EXE.
    {
        println!("");
        println!("Attempting to write patched data back to {}", exe_name);
        let mut outfile = File::create(&exe_name)?;
        // Write the DOS stub back out
        outfile.write_all(&binary[..actual_mz_size])?;
        // And the actual PMW1 exe!
        outfile.write_all(&pmw1_exe.as_bytes())?;
        println!("Patching successful!");
    }

    Ok(())
}
