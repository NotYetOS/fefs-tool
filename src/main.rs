use std::sync::Arc;
use std::sync::Mutex;
use fefs::device::BlockDevice;
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

const BLOCK_SIZE: usize = 512;

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
    println!("{:?}", root_dir.ls());
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
    let mut file = dir.create_file("tlnb").unwrap();
    assert!(dir.exist("tlnb"));

    let mut buf = [0; 10];
    let str_len = "hello fefs abc".len();
    file.write("hello fefs abc".as_bytes(), WriteType::OverWritten).unwrap();
    let len = file.read(&mut buf).unwrap();
    let ret = core::str::from_utf8(&buf[0..len]).unwrap();
    assert_eq!(ret, "hello fefs");
    println!("{}", ret);

    file.seek(6).unwrap();
    let len = file.read(&mut buf).unwrap();
    let ret = core::str::from_utf8(&buf[0..len]).unwrap();
    assert_eq!(ret, "fefs abc");

    file.seek(str_len).unwrap();
    let len = file.read(&mut buf).unwrap();
    let ret = core::str::from_utf8(&buf[0..len]).unwrap();
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
