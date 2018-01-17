// Copyright 2018 the Tectonic Project
// Licensed under the MIT License.

//! Note: we need to store this code as `tests/util/mod.rs` rather than
//! `tests/util.rs` because otherwise Cargo thinks it is a test executable of
//! its own.

// An item is considered unused if at least one testing binary
// has no reference to it. This yields a lot of false-positives
// using this testing setup...
#![allow(dead_code)]

use flate2::read::GzDecoder;
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tectonic::io::{FilesystemIo, GenuineStdoutIo, IoStack, MemoryIo, IoProvider, InputHandle, OutputHandle, OpenResult};
use tectonic::io::local_cache::LocalCache;
use tectonic::io::itarbundle::{HttpITarIoFactory, ITarBundle};
use tectonic::io::testing::SingleInputFileIo;
use tectonic::status::{NoopStatusBackend, StatusBackend};

const TOP: &str = env!("CARGO_MANIFEST_DIR");
const BUNDLE_URL: &str = "https://dl.bintray.com/pkgw/tectonic/tl2016extras/2016.0r4/tlextras-2016.0r4.tar";

// This allows the user to pull in new assets and modify the 'static' bundle.
// Simply run `env STATIC_BUNDLE_BACKEND="network bundle"`.
fn is_testing_for_real() -> bool {
    env::var("STATIC_BUNDLE_BACKEND") != Ok("network bundle".to_owned())
}


pub fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut f = File::open(&path).unwrap();
    f.read_to_end(&mut buffer).unwrap();
    buffer
}

pub fn test_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(TOP);
    path.push("tests");
    path.push(parts.iter().collect::<PathBuf>());
    path
}


// Duplicated from Cargo's own testing code:
// https://github.com/rust-lang/cargo/blob/19fdb308/tests/cargotest/support/mod.rs#L305-L318
pub fn cargo_dir() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe()
                .ok()
                .map(|mut path| {
                         path.pop();
                         if path.ends_with("deps") {
                             path.pop();
                         }
                         path
                     })
        })
        .unwrap_or_else(|| panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test"))
}


/// Convenience structure for comparing expected and actual output in various
/// tests.
pub struct ExpectedInfo {
    name: OsString,
    contents: Vec<u8>,
    gzipped: bool,
}

impl ExpectedInfo {
    pub fn read<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let name = path.file_name().unwrap().to_owned();

        let mut f = File::open(path).unwrap();
        let mut contents = Vec::new();
        f.read_to_end(&mut contents).unwrap();

        ExpectedInfo { name: name, contents: contents, gzipped: false }
    }

    pub fn read_with_extension(pbase: &mut PathBuf, extension: &str) -> Self {
        pbase.set_extension(extension);
        Self::read(pbase)
    }

    pub fn read_with_extension_gz(pbase: &mut PathBuf, extension: &str) -> Self {
        pbase.set_extension(extension);
        let name = pbase.file_name().unwrap().to_owned();

        let mut dec = GzDecoder::new(File::open(pbase).unwrap());
        let mut contents = Vec::new();
        dec.read_to_end(&mut contents).unwrap();

        ExpectedInfo { name: name, contents: contents, gzipped: true }
    }

    pub fn test_data(&self, observed: &Vec<u8>) {
        if &self.contents == observed {
            return;
        }

        // For nontrivial tests, it's really tough to figure out what
        // changed without being able to do diffs, etc. So, write out the
        // buffers.
        {
            let mut n = self.name.clone();
            n.push(".expected");
            let mut f = File::create(&n).expect(&format!("failed to create {} for test failure diagnosis", n.to_string_lossy()));
            f.write_all(&self.contents).expect(&format!("failed to write {} for test failure diagnosis", n.to_string_lossy()));
        }
        {
            let mut n = self.name.clone();
            n.push(".observed");
            let mut f = File::create(&n).expect(&format!("failed to create {} for test failure diagnosis", n.to_string_lossy()));
            f.write_all(observed).expect(&format!("failed to write {} for test failure diagnosis", n.to_string_lossy()));
        }
        panic!("difference in {}; contents saved to disk", self.name.to_string_lossy());
    }

    pub fn test_from_collection(&self, files: &HashMap<OsString, Vec<u8>>) {
        if !self.gzipped {
            self.test_data(files.get(&self.name).unwrap())
        } else {
            let mut buf = Vec::new();
            let mut dec = GzDecoder::new(&files.get(&self.name).unwrap()[..]);
            dec.read_to_end(&mut buf).unwrap();
            self.test_data(&buf);
        }
    }
}

pub struct TestBundle {
    pub status_backend: NoopStatusBackend,
    pub io: Vec<Box<IoProvider>>,
    pub mem_io: MemoryIo,
}

impl TestBundle {
    pub fn new() -> Self {
        let genuine_stdout = GenuineStdoutIo::new();
        TestBundle {
            status_backend: NoopStatusBackend::new(),
            io: vec![
                Box::new(genuine_stdout),
            ],
            mem_io: MemoryIo::new(true),
        }
    }

    pub fn with_file(self, p: &Path) -> Self {
        self.with_io(SingleInputFileIo::new(p))
    }

    pub fn with_folder(self, p: &Path) -> Self {
        self.with_io(FilesystemIo::new(p, false, false, HashSet::new()))
    }

    pub fn with_static_bundle(self) -> Self {
        let itb = ITarBundle::<HttpITarIoFactory>::new(BUNDLE_URL);

        let mut url2digest_path = test_path(&["test_bundle", "urls"]);
        url2digest_path.push(
            BUNDLE_URL.chars().filter(|c| c.is_alphanumeric()).collect::<String>()
        );

        if is_testing_for_real() {
            self.with_io(LocalCache::<PanickyIoProvider>::new(
                PanickyIoProvider,
                &url2digest_path,
                &test_path(&["test_bundle", "manifests"]),
                &test_path(&["test_bundle", "formats"]),
                &test_path(&["test_bundle", "files"]),
                &mut NoopStatusBackend::new()
            ).expect("Unable to initialize LocalCache"))
        } else {
            self.with_io(LocalCache::<ITarBundle<HttpITarIoFactory>>::new(
                itb,
                &url2digest_path,
                &test_path(&["test_bundle", "manifests"]),
                &test_path(&["test_bundle", "formats"]),
                &test_path(&["test_bundle", "files"]),
                &mut NoopStatusBackend::new()
            ).expect("Unable to initialize LocalCache"))
        }
    }

    pub fn with_io<T: IoProvider + 'static>(mut self, io: T) -> Self {
        self.io.push(Box::new(io));
        self
    }

    pub fn as_iostack(&mut self) -> IoStack {
        let mut ios = vec![&mut self.mem_io as &mut IoProvider];
        for io in &mut self.io {
            ios.push(&mut **io);
        }
        IoStack::new(ios)
    }
}

struct PanickyIoProvider;
impl IoProvider for PanickyIoProvider {
    fn output_open_name(&mut self, name: &OsStr) -> OpenResult<OutputHandle> {
        panic!("Not known to static bundle: {:?}", name)
    }

    fn input_open_name(&mut self, name: &OsStr, _: &mut StatusBackend) -> OpenResult<InputHandle> {
        panic!("Not known to static bundle {:?}", name)
    }
}


pub fn assert_file_eq(name: &OsStr, expected: &Vec<u8>, observed: &Vec<u8>) {
    if expected == observed {
        return;
    }

    // For nontrivial tests, it's really tough to figure out what
    // changed without being able to do diffs, etc. So, write out the
    // buffers.

    {
        let mut n = name.to_owned();
        n.push(".expected");
        let mut f = File::create(&n).expect(&format!("failed to create {} for test failure diagnosis", n.to_string_lossy()));
        f.write_all(expected).expect(&format!("failed to write {} for test failure diagnosis", n.to_string_lossy()));
    }
    {
        let mut n = name.to_owned();
        n.push(".observed");
        let mut f = File::create(&n).expect(&format!("failed to create {} for test failure diagnosis", n.to_string_lossy()));
        f.write_all(observed).expect(&format!("failed to write {} for test failure diagnosis", n.to_string_lossy()));
    }

    panic!("difference in {}; contents saved to disk", name.to_string_lossy());
}
