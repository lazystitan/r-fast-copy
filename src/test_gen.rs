use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs::{create_dir_all, File};
use std::io;
use std::io::Write;
use std::path::Path;

pub struct TestDirGenerator<'a> {
    threshold: f64,
    max_depth: u32,
    upper_path: &'a Path,
}

impl<'a> TestDirGenerator<'a> {
    pub fn builder() -> TestBirGeneratorBuilder<'a> {
        TestBirGeneratorBuilder::new()
    }

    pub fn gen(self) -> Result<(), io::Error> {
        self.gen_dir_recursive(self.upper_path.to_str().unwrap(), 0)?;
        Ok(())
    }

    fn gen_dir_recursive(&self, upper_path: &str, mut level: u32) -> Result<(), io::Error> {
        level += 1;
        let mut i = 0;

        while self.if_continue_gen_file() {
            let file_name = Self::gen_string(6);
            let upper = upper_path.to_string();
            let full_path = upper + "/" + &file_name + &i.to_string();
            let mut file = File::create(Path::new(&full_path))?;
            file.write_all(Self::gen_string(100).as_bytes())?;
            println!("Created file at {}", full_path);
            i += 1;
        }

        if level >= self.max_depth {
            return Ok(());
        }

        while self.if_continue_gen(level) {
            let dir_name = Self::gen_string(6);
            let upper = upper_path.to_string();
            let full_path = upper + "/" + &dir_name + &i.to_string();
            let new_path_prefix = Path::new(&full_path);
            create_dir_all(new_path_prefix)?;
            self.gen_dir_recursive(&full_path, level)?;
            println!("Created directory at {}", full_path);
            i += 1;
        }

        Ok(())
    }

    fn if_continue_gen(&self, level: u32) -> bool {
        let mut i: f64 = thread_rng().gen_range(0.0..1.0);
        //the lesser the level is,  the return tends to true
        i = i * (self.max_depth - level) as f64;
        i > self.threshold
    }

    fn if_continue_gen_file(&self) -> bool {
        let i: f64 = thread_rng().gen_range(0.0..1.0);
        i > self.threshold
    }

    fn gen_string(len: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }
}

pub struct TestBirGeneratorBuilder<'a> {
    threshold: f64,
    max_depth: u32,
    upper_path: Option<&'a Path>,
}

impl<'a> TestBirGeneratorBuilder<'a> {
    fn new() -> Self {
        Self {
            threshold: 0.2,
            max_depth: 5,
            upper_path: None,
        }
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn upper_path(mut self, upper_path: &'a Path) -> Self {
        self.upper_path = Some(upper_path);
        self
    }

    pub fn build(self) -> Result<TestDirGenerator<'a>, &'static str> {
        let Self {
            threshold,
            max_depth,
            upper_path,
        } = self;

        if let Some(upper_path) = upper_path {
            Ok(TestDirGenerator {
                threshold,
                max_depth,
                upper_path,
            })
        } else {
            Err("upper_path not set")
        }
    }
}
