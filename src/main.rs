use std::sync::Arc;
use std::sync::Mutex;
use fefs::system::FileSystem;
use std::io::{
    Read,
    Seek, 
    SeekFrom, 
    Write
};
use std::fs::{
    File, 
    OpenOptions
};
use fefs::{
    device::BlockDevice, 
    file::WriteType
};

const BLOCK_SIZE: usize = 512;
const USER_BINS_RECORD: &'static str = "../user/bins";
const USER_BINS_PATH: &'static str = "../user/target/riscv64gc-unknown-none-elf/debug/";

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read(&self, addr: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(addr as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SIZE, "Not a complete block!");
    }

    fn write(&self, addr: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(addr as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SIZE, "Not a complete block!");
    }
}

fn main() -> std::io::Result<()> {
    if let Ok(_) =  File::open("fs.img") {
        std::fs::remove_file("fs.img").unwrap();
    }

    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("fs.img")?;
        f.set_len(8192 * 512).unwrap();
        f
    })));

    let fs = FileSystem::create(
        block_file, 
        BLOCK_SIZE, 
        8
    );

    let fs = fs.lock();
    let mut root_dir = fs.root();
    root_dir.mkdir("bin").unwrap();
    let mut bin_dir = root_dir.cd("bin").unwrap();

    let mut bins_record = Vec::new();
    let mut bin_record = File::open(USER_BINS_RECORD).unwrap();
    bin_record.read_to_end(&mut bins_record).unwrap();
    let bins_str = String::from_utf8(bins_record).unwrap();
    
    let mut buf = Vec::new();
    let mut buf_test = Vec::new();

    for bin_str in bins_str.split("\n") {
        if bin_str.is_empty() { break; }
        let mut f = File::open(format!("{}{}", USER_BINS_PATH, bin_str)).unwrap();
        let len = f.read_to_end(&mut buf).unwrap();

        let mut bin= bin_dir.create_file(bin_str).unwrap();
        bin.write(&buf[0..len], WriteType::OverWritten).unwrap();
        bin.read_to_vec(&mut buf_test).unwrap();
        assert_eq!(buf[0..len], buf_test[0..len]);
    }

    println!("{:#?}", bin_dir.ls());
    Ok(())
}

#[test]
fn fefs_test() -> std::io::Result<()> {
    use fefs::dir::DirError;
    use fefs::system::FileSystem;
    use std::fs::{
        File, 
        OpenOptions
    };
    use std::sync::{
        Arc,
        Mutex
    };
    use fefs::file::{
        WriteType,
        FileError
    };

    if let Ok(_) =  File::open("fs.img") {
        std::fs::remove_file("fs.img").unwrap();
    }

    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("fs.img")?;
        f.set_len(8192 * 512).unwrap();
        f
    })));

    let fs = FileSystem::create(
        block_file, 
        BLOCK_SIZE, 
        8
    );

    let fs = fs.lock();
    let mut root = fs.root();

    root.mkdir("fefs").unwrap();
    assert_eq!(root.mkdir("fefs").err().unwrap(), DirError::DirExist);
    let mut dir = root.cd("fefs").unwrap();
    let mut file =  dir.create_file("tlnb").unwrap();
    assert!(dir.exist("tlnb"));

    let mut buf = [0; 10];
    let mut vec_buf = Vec::new();

    let str_len = "hello fefs abc".len();
    file.write("hello fefs abc".as_bytes(), WriteType::OverWritten).unwrap();
    let len = file.read(&mut buf).unwrap();
    let ret = core::str::from_utf8(&buf[0..len]).unwrap();
    assert_eq!(ret, "hello fefs");
    println!("{}", ret);

    file.seek(6).unwrap();
    let len = file.read_to_vec(&mut vec_buf).unwrap();
    let ret = core::str::from_utf8(&vec_buf[0..len]).unwrap();
    assert_eq!(ret, "fefs abc");

    file.seek(str_len).unwrap();
    let len = file.read_to_vec(&mut vec_buf).unwrap();
    let ret = core::str::from_utf8(&vec_buf[0..len]).unwrap();
    assert_eq!(ret, "");
    assert_eq!(file.seek(str_len + 1).err().unwrap(), FileError::SeekValueOverFlow);

    println!("{:#?}", dir.ls());

    root.delete("fefs").unwrap();
    assert!(!root.exist("fefs"));
    assert_eq!(root.delete("fefs").err().unwrap(), DirError::NotFound);
    assert!(root.ls().len() == 0);

    root.mkdir("fefs").unwrap();
    println!("{:#?}", root.ls());

    std::fs::remove_file("fs.img").unwrap();

    Ok(())
}
