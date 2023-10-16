// Copyright (c) 2023 The Nimbus Authors. All rights reserved.
//
// The use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::io::{Error as IOError, ErrorKind as IOErrorKind, Read, Seek, SeekFrom, Write};

use crate::{Manifest, Section, Trailer};

const MAGIC_NUMBER: &[u8; 8] = b"k1ck0ff!";

const TRAILER_SIZE: usize = std::mem::size_of::<Trailer>();

pub fn read_manifest<T>(reader: &mut T) -> Result<Manifest, IOError>
where
    T: Read + Seek,
{
    let trailer = read_trailer(reader)?;

    let size = trailer.manifest.len as usize;
    let mut buf = vec![0; size];

    reader.seek(SeekFrom::Start(trailer.manifest.pos))?;
    reader.read_exact(&mut buf)?;

    match serde_json::from_slice(&buf) {
        Ok(manifest) => Ok(manifest),
        Err(err) => Err(IOError::new(IOErrorKind::InvalidData, err)),
    }
}

pub fn write_manifest<T>(writer: &mut T, manifest: &Manifest) -> Result<(), IOError>
where
    T: Write + Seek,
{
    let w_len = writer.seek(SeekFrom::Current(0))?;

    let raw_manifest = match serde_json::to_vec(&manifest) {
        Ok(v) => v,
        Err(err) => return Err(IOError::new(IOErrorKind::InvalidData, err)),
    };

    let runtime_pos = 0;
    let runtime_len = w_len;

    let manifest_pos = w_len;
    let manifest_len = raw_manifest.len() as u64;

    let trailer = Trailer {
        magic: *MAGIC_NUMBER,
        runtime: Section {
            pos: runtime_pos,
            len: runtime_len,
        },
        manifest: Section {
            pos: manifest_pos,
            len: manifest_len,
        },
    };

    writer.write(&raw_manifest[..])?;
    write_trailer(writer, &trailer)?;

    Ok(())
}

pub fn read_trailer<T>(reader: &mut T) -> Result<Trailer, IOError>
where
    T: Read + Seek,
{
    let mut trailer = Trailer::default();

    // Casting from "usize" to "i64" will no longer be safe if the
    // "Trailer" size grows beyond "i64::MAX". This is unlikely to
    // happen in practice. Thus, the current code should be enough
    // for foreseeable future.
    reader.seek(SeekFrom::End(-(TRAILER_SIZE as i64)))?;

    reader.read_exact(&mut trailer.magic)?;

    trailer.runtime.pos = read_u64(reader)?;
    trailer.runtime.len = read_u64(reader)?;

    trailer.manifest.pos = read_u64(reader)?;
    trailer.manifest.len = read_u64(reader)?;

    // Check that the obtained magic number matches the expected one.
    // Otherwiwe, the trailer structure may have been populated with
    // meaningless data.
    match trailer.magic == MAGIC_NUMBER[..] {
        true => Ok(trailer),
        false => Err(IOError::from(IOErrorKind::InvalidData)),
    }
}

pub fn write_trailer<T>(writer: &mut T, trailer: &Trailer) -> Result<(), IOError>
where
    T: Write,
{
    writer.write(&trailer.magic)?;

    writer.write(&trailer.runtime.pos.to_ne_bytes())?;
    writer.write(&trailer.runtime.len.to_ne_bytes())?;

    writer.write(&trailer.manifest.pos.to_ne_bytes())?;
    writer.write(&trailer.manifest.len.to_ne_bytes())?;

    Ok(())
}

fn read_u64<T>(reader: &mut T) -> Result<u64, IOError>
where
    T: Read,
{
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;

    Ok(u64::from_ne_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, io::Cursor as IOCursor};

    #[test]
    fn read_manifest_when_valid_then_succeeds() {
        let expected = Manifest {
            argv: vec![String::from("foo"), String::from("bar")],
            env: HashMap::from([(String::from("SOME_KEY"), String::from("some-value"))]),
        };

        let raw_manifest = serde_json::to_string(&expected).unwrap();
        let runtime = (0..1024).map(|_| rand::random::<u8>()).collect::<Vec<_>>();

        let mut data = Vec::new();
        data.extend_from_slice(&runtime);
        data.extend_from_slice(raw_manifest.as_bytes());

        data.extend_from_slice(MAGIC_NUMBER);

        data.extend_from_slice(&0_u64.to_ne_bytes());
        data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());

        data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());
        data.extend_from_slice(&(raw_manifest.len() as u64).to_ne_bytes());

        let reader = &mut IOCursor::new(data);

        let actual = read_manifest(reader).unwrap();

        assert_eq!(actual.argv, expected.argv);
        assert_eq!(actual.env, expected.env);
    }

    #[test]
    fn read_manifest_when_not_valid_then_fails() {
        let runtime = (0..1024).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
        let manifest = (0..1024).map(|_| rand::random::<u8>()).collect::<Vec<_>>();

        let mut data = Vec::new();
        data.extend_from_slice(&runtime);
        data.extend_from_slice(&manifest);

        data.extend_from_slice(MAGIC_NUMBER);

        data.extend_from_slice(&0_u64.to_ne_bytes());
        data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());

        data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());
        data.extend_from_slice(&(manifest.len() as u64).to_ne_bytes());

        let reader = &mut IOCursor::new(data);

        assert!(read_manifest(reader).is_err());
    }

    #[test]
    fn write_manifest_when_valid_then_succeeds() {
        let manifest = Manifest {
            argv: vec![String::from("foo"), String::from("bar")],
            env: HashMap::from([(String::from("SOME_KEY"), String::from("some-value"))]),
        };

        let runtime = (0..1024).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
        let raw_manifest = "{\"argv\":[\"foo\",\"bar\"],\"env\":{\"SOME_KEY\":\"some-value\"}}";

        let writer = &mut IOCursor::new(runtime.clone());

        writer.seek(SeekFrom::End(0)).unwrap();
        write_manifest(writer, &manifest).unwrap();

        let mut want_data = Vec::new();
        want_data.extend_from_slice(&runtime);
        want_data.extend_from_slice(raw_manifest.as_bytes());
        want_data.extend_from_slice(MAGIC_NUMBER);
        want_data.extend_from_slice(&0_u64.to_ne_bytes());
        want_data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());
        want_data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());
        want_data.extend_from_slice(&(raw_manifest.len() as u64).to_ne_bytes());

        assert_eq!(want_data, writer.get_ref().clone());
    }

    #[test]
    fn read_trailer_when_valid_then_succeeds() {
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC_NUMBER);

        data.extend_from_slice(&0_u64.to_ne_bytes());
        data.extend_from_slice(&99_u64.to_ne_bytes());

        data.extend_from_slice(&100_u64.to_ne_bytes());
        data.extend_from_slice(&120_u64.to_ne_bytes());

        let trailer = read_trailer(&mut IOCursor::new(data)).unwrap();

        assert_eq!(trailer.magic, MAGIC_NUMBER[..]);

        assert_eq!(trailer.runtime.pos, 0);
        assert_eq!(trailer.runtime.len, 99);

        assert_eq!(trailer.manifest.pos, 100);
        assert_eq!(trailer.manifest.len, 120);
    }

    #[test]
    fn read_trailer_when_not_valid_then_fails() {
        let data = (0..1024).map(|_| rand::random::<u8>()).collect::<Vec<_>>();
        assert!(read_trailer(&mut IOCursor::new(data)).is_err());
    }

    #[test]
    fn write_trailer_when_valid_then_succeeds() {
        let runtime = (0..1024).map(|_| rand::random::<u8>()).collect::<Vec<_>>();

        let trailer = Trailer {
            magic: *MAGIC_NUMBER,
            runtime: Section {
                pos: 0,
                len: runtime.len() as u64,
            },
            manifest: Section {
                pos: runtime.len() as u64,
                len: 0,
            },
        };

        let writer = &mut IOCursor::new(runtime.clone());

        writer.seek(SeekFrom::End(0)).unwrap();
        write_trailer(writer, &trailer).unwrap();

        let mut want_data = Vec::new();
        want_data.extend_from_slice(&runtime);
        want_data.extend_from_slice(MAGIC_NUMBER);
        want_data.extend_from_slice(&0_u64.to_ne_bytes());
        want_data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());
        want_data.extend_from_slice(&(runtime.len() as u64).to_ne_bytes());
        want_data.extend_from_slice(&0_u64.to_ne_bytes());

        assert_eq!(want_data, writer.get_ref().clone());
    }

    #[test]
    fn manifest_serde_rountrip() {
        let rw = &mut IOCursor::new(Vec::new());

        let want = Manifest {
            argv: vec![String::from("foo"), String::from("bar")],
            env: HashMap::from([(String::from("SOME_KEY"), String::from("some-value"))]),
        };

        write_manifest(rw, &want).unwrap();

        assert_eq!(want, read_manifest(rw).unwrap());
    }
}
