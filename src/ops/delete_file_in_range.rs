// `DeleteFilesInRange` can be used prior to compacting the deleted range as long
// as snapshot readers do not need to access them. It drops files that are
// completely contained in the deleted range. That saves write-amp because, in
// `CompactRange`, the file data would have to be rewritten several times before it
// reaches the bottom of the LSM, where tombstones can finally be dropped.

use ffi;
use libc::{c_char, size_t};

use crate::{handle::Handle, ColumnFamily, Error};

pub trait DeleteFileInRange {
    fn delete_file_in_range<K>(&self, start_key: K, limit_key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>;
}

impl<T> DeleteFileInRange for T
where
    T: DeleteFileInRangeCF,
{
    fn delete_file_in_range<K>(&self, start_key: K, limit_key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        self.delete_file_in_range_cf_full(None, start_key, limit_key)
    }
}

pub trait DeleteFileInRangeCF {
    fn delete_file_in_range_cf<K>(
        &self,
        cf: &ColumnFamily,
        start_key: K,
        limit_key: K,
    ) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        self.delete_file_in_range_cf_full(Some(cf), start_key, limit_key)
    }

    fn delete_file_in_range_cf_full<K>(
        &self,
        cf: Option<&ColumnFamily>,
        start_key: K,
        limit_key: K,
    ) -> Result<(), Error>
    where
        K: AsRef<[u8]>;
}

impl<T> DeleteFileInRangeCF for T
where
    T: Handle<ffi::rocksdb_t> + super::Write,
{
    fn delete_file_in_range_cf_full<K>(
        &self,
        cf: Option<&ColumnFamily>,
        start_key: K,
        limit_key: K,
    ) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        let start_key = start_key.as_ref();
        let start_key_ptr = start_key.as_ptr() as *const c_char;
        let start_key_len = start_key.len();

        let limit_key = limit_key.as_ref();
        let limit_key_ptr = limit_key.as_ptr() as *const c_char;
        let limit_key_len = limit_key.len();

        unsafe {
            match cf {
                Some(cf) => ffi_try!(ffi::rocksdb_delete_file_in_range_cf(
                    self.handle(),
                    cf.handle(),
                    start_key_ptr,
                    start_key_len,
                    limit_key_ptr,
                    limit_key_len,
                )),
                None => ffi_try!(ffi::rocksdb_delete_file_in_range(
                    self.handle(),
                    start_key_ptr,
                    start_key_len,
                    limit_key_ptr,
                    limit_key_len,
                )),
            }

            Ok(())
        }
    }
}
