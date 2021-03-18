use std::fs::File;
use std::io::{
    Read,
    Seek, 
    SeekFrom, 
    Write
};
use std::sync::Mutex;
use fefs::device::BlockDevice;


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

fn main() {}

#[test]
fn fefs_test() -> std::io::Result<()> {
    use std::fs::{
        File, 
        OpenOptions
    };

    use std::sync::{
        Arc,
        Mutex
    };

    use fefs::{
        dir::DirError, 
        file::WriteType
    };

    use fefs::system::FileSystem;

    if let Ok(_) =  File::open("target/fs.img") {
        std::fs::remove_file("target/fs.img").unwrap();
    }

    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
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

    root_dir.mkdir("fefs").unwrap();
    assert_eq!(root_dir.mkdir("fefs").err().unwrap(), DirError::DirExist);
    let mut dir = root_dir.cd("fefs").unwrap();
    let mut file = dir.create_file("tlnb").unwrap();

    file.write("hello fefs".as_bytes(), WriteType::OverWritten).unwrap();
    let mut buf = [0; BLOCK_SIZE];
    file.read(&mut buf).unwrap();
    let ret = core::str::from_utf8(&buf[0.."hello fefs".len()]).unwrap();
    assert_eq!(ret, "hello fefs");
    println!("{}", ret);
    println!("{:#?}", dir.ls());

    root_dir.delete("fefs").unwrap();
    assert_eq!(root_dir.delete("fefs").err().unwrap(), DirError::NotFound);
    assert!(root_dir.ls().len() == 0);

    root_dir.mkdir("fefs").unwrap();
    println!("{:#?}", root_dir.ls());

    std::fs::remove_file("target/fs.img").unwrap();

    Ok(())
}