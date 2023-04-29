use std::{convert::TryFrom, fs::File, io::Seek, path::{Path, PathBuf}, io, fs};

fn main() {
    
    let mut args = std::env::args();
    let _exe_name = args.next().expect("Failed To Skip EXE Name");
    let efi_path = PathBuf::from(args.next().expect("Path To '.efi' Files Must Be Given As Arguement"));
    let fat_path = efi_path.with_extension("fat");
    let disk_path = fat_path.with_extension("gdt");

    create_fat_filesystem(&fat_path, &efi_path);
    create_gpt_disk(&disk_path, &fat_path);


}

fn create_fat_filesystem(fat_path: &Path, efi_file: &Path) {
    // Get Size Of EFI File
    let efi_size = fs::metadata(&efi_file).expect("Failed To Get EFI Size").len();
    // Megabyte
    let mb = 1024 * 1024;
    // Round Size To Next Megabyte
    let efi_size_rounded = ((efi_size - 1) / mb + 1) * mb;


    // Set Options For Where To Make The ISO
    let fat_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&fat_path)
        .expect("Failed To Set ISO Options");
    // Set Length To Rounded Up Size Of EFI File
    fat_file.set_len(efi_size_rounded).expect("Failed To Set Length Of fat_file");

    // Create New FAT File System With File Options
    let format_options = fatfs::FormatVolumeOptions::new();
    fatfs::format_volume(&fat_file, format_options).expect("Failed To Format Volume");
    // Open The FAT File System
    let filesystem = fatfs::FileSystem::
        new(&fat_file, fatfs::FsOptions::new()).expect("Failed To Open FAT File System");

    // Create Dirs And Empty File
    let root_dir = filesystem.root_dir();
    root_dir.create_dir("efi").expect("Failed To Make efi Dir");
    root_dir.create_dir("efi/boot").expect("Failed To Make efi/boot");
    let mut bootx64 = root_dir.create_file("efi/boot/bootx64.efi").expect("Failed To Create bootx64.efi");
    bootx64.truncate().expect("Failed To Truncate bootx64.efi");
    // Copy EFI File Into bootx64.efi
    io::copy(&mut fs::File::open(&efi_file).expect("Failed To Open EFI File"), &mut bootx64).expect("Failed To Copy EFI Into bootx64.efi");


}


fn create_gpt_disk(disk_path: &Path, fat_image: &Path) {
    // Create New Disk File
    let mut disk = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&disk_path)
        .expect("Failed To Create New Disk File");

    // Set Size Of File

    let partition_size: u64 = fs::metadata(&fat_image).expect("Failed To Get FAT_image MetaData").len();
    let disk_size = partition_size + 1024 * 64; // For GPT Headers
    disk.set_len(disk_size).expect("Failed To Set Disk Size");


    // Create A Protective MBR At LBA0 So The Disk Is Not Considered
    // Unformatted On BIOS Systems
    let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
        u32::try_from((disk_size / 512) - 1).unwrap_or(0xFF_FF_FF_FF),
    );

    mbr.overwrite_lba0(&mut disk).expect("Failed To Write MBR");

    // Create New GPT Structure
    let block_size = gpt::disk::LogicalBlockSize::Lb512;
    let mut gpt = gpt::GptConfig::new()
        .writable(true)
        .initialized(false)
        .logical_block_size(block_size)
        .create_from_device(Box::new(&mut disk), None)
        .expect("Failed To Make GPT Structure");
    gpt.update_partitions(Default::default()).expect("Failed To Update Partitions");

    // Add New EFI System Partition And Get Its Byte Offset In File
    let partition_id = gpt.add_partition(
        "boot", 
        partition_size, 
        gpt::partition_types::EFI, 
        0, 
        None,
    ).expect("Failed To Add New EFI System Partition");
    let partition = gpt.partitions().get(&partition_id).expect("Failed To Get Partition From ID");
    let start_offset = partition.bytes_start(block_size).expect("Failed To Get Start Offset");

    // Write Changes And Close GPT
    gpt.write().expect("Failed To Write Changes To GPT");

    // Place Fat File System In The New Partition
    disk.seek(io::SeekFrom::Start(start_offset)).expect("Failed To Seek"); // Sets Pos Of Disk
    io::copy(&mut File::open(&fat_image).expect("Failed To Find Fat Image"), &mut disk).expect("Failed To Place FAT File System In New Partition");


}