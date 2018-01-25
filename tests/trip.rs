// tests/trip.rs - implemention the TRIP test for Tectonic
// Copyright 2016-2018 the Tectonic Project
// Licensed under the MIT License.

/// Our incarnation of the classic TRIP test. Unfortunately, the test is
/// defined in terms of the precise terminal output and error handling behavior
/// of the engine, so you can't do anything to improve the (incredibly poor) UX
/// of the TeX engine without having to fudge what "the TRIP test" is. That is
/// what we have done.
///
/// Cargo tries to run tests in multiple simultaneous threads, which of course
/// totally fails for Tectonic since the engine has tons of global state. The
/// multithreading can be disabled by setting the RUST_TEST_THREADS environment
/// variable to "1", but that's an annoying solution. So, we use a global mutex
/// to achieve the same effect. Classy.

extern crate flate2;
#[macro_use] extern crate lazy_static;
extern crate tectonic;

use std::ffi::OsStr;
use std::sync::Mutex;

use tectonic::engines::NoopIoEventBackend;
use tectonic::io::{FilesystemPrimaryInputIo, IoProvider, IoStack, MemoryIo};
use tectonic::io::testing::SingleInputFileIo;
use tectonic::status::NoopStatusBackend;
use tectonic::TexEngine;

mod util;
use util::{ExpectedInfo, test_path};
const TOP: &str = env!("CARGO_MANIFEST_DIR");

lazy_static! {
    static ref LOCK: Mutex<u8> = Mutex::new(0u8);
}


#[test]
fn trip_test() {
    let _guard = LOCK.lock().unwrap(); // until we're thread-safe ...

    let mut p = test_path(&["trip", "trip"]);

    // An IoProvider for the input file.
    p.set_extension("tex");
    let mut tex = FilesystemPrimaryInputIo::new(&p);

    // And the TFM file.
    p.set_extension("tfm");
    let mut tfm = SingleInputFileIo::new(&p);

    // Read in the expected outputs.
    let expected_log = ExpectedInfo::read_with_extension(&mut p, "log");
    let expected_xdv = ExpectedInfo::read_with_extension(&mut p, "xdv");
    let expected_fot = ExpectedInfo::read_with_extension(&mut p, "fot");
    p.set_file_name("tripos");
    let expected_os = ExpectedInfo::read_with_extension(&mut p, "tex");

    // MemoryIo layer that will accept the outputs. Save `files` since the
    // engine consumes `mem`.
    let mut mem = MemoryIo::new(true);

    // First engine pass -- make the format file.
    {
        let mut io = IoStack::new(vec![
            &mut mem as &mut IoProvider,
            &mut tex,
            &mut tfm,
        ]);
        TexEngine::new()
            .halt_on_error_mode(false)
            .initex_mode(true)
            .process(&mut io, &mut NoopIoEventBackend::new(),
                      &mut NoopStatusBackend::new(), "INITEX", "trip").unwrap();
    }

    // Second pass -- process it
    {
        let mut io = IoStack::new(vec![
            &mut mem as &mut IoProvider,
            &mut tex,
            &mut tfm,
        ]);
        TexEngine::new()
            .halt_on_error_mode(false)
            .initex_mode(false)
            .process(&mut io, &mut NoopIoEventBackend::new(),
                      &mut NoopStatusBackend::new(), "trip.fmt.gz", "trip").unwrap();
    }

    // Check that outputs match expectations.
    let files = &*mem.files.borrow();
    expected_log.test_from_collection(files);
    expected_xdv.test_from_collection(files);
    expected_os.test_from_collection(files);
    expected_fot.test_data(files.get(OsStr::new("")).unwrap());
}


#[test]
fn etrip_test() {
    let _guard = LOCK.lock().unwrap(); // until we're thread-safe ...

    let mut p = test_path(&["trip", "etrip"]);

    // An IoProvider the input file.
    p.set_extension("tex");
    let mut tex = FilesystemPrimaryInputIo::new(&p);

    // And the TFM file.
    p.set_extension("tfm");
    let mut tfm = SingleInputFileIo::new(&p);

    // Read in the expected outputs.
    let expected_log = ExpectedInfo::read_with_extension(&mut p, "log");
    let expected_xdv = ExpectedInfo::read_with_extension(&mut p, "xdv");
    let expected_fot = ExpectedInfo::read_with_extension(&mut p, "fot");
    let expected_out = ExpectedInfo::read_with_extension(&mut p, "out");

    // MemoryIo layer that will accept the outputs. Save `files` since the
    // engine consumes `mem`.
    let mut mem = MemoryIo::new(true);
    let files = mem.files.clone();

    // First engine pass -- make the format file.
    {
        let mut io = IoStack::new(vec![
            &mut mem as &mut IoProvider,
            &mut tex,
            &mut tfm,
        ]);
        TexEngine::new()
            .halt_on_error_mode(false)
            .initex_mode(true)
            .process(&mut io, &mut NoopIoEventBackend::new(),
                      &mut NoopStatusBackend::new(), "INITEX", "etrip").unwrap();
    }

    // Second pass -- process it
    {
        let mut io = IoStack::new(vec![
            &mut mem,
            &mut tex,
            &mut tfm,
        ]);
        TexEngine::new()
            .halt_on_error_mode(false)
            .initex_mode(false)
            .process(&mut io, &mut NoopIoEventBackend::new(),
                      &mut NoopStatusBackend::new(), "etrip.fmt.gz", "etrip").unwrap();
    }

    // Check that outputs match expectations.
    let files = &*files.borrow();
    expected_log.test_from_collection(files);
    expected_xdv.test_from_collection(files);
    expected_out.test_from_collection(files);
    expected_fot.test_data(files.get(OsStr::new("")).unwrap());
}
