//! Zipfile Virtual Table.
//!
//! Port of [Zipfile](https://sqlite.org/src/file/ext/misc/zipfile.c) C
//! extension: `https://www.sqlite.org/zipfile.html`

use crate::types::ValueRef;
use crate::vtab::{
    dequote, Context, CreateVTab, Filters, IndexConstraintOp, IndexInfo, Inserts, Module,
    TransactionVTab, UpdateVTab, Updates, VTab, VTabConfig, VTabConnection, VTabCursor, VTabKind,
};
use crate::{ffi, Connection, Error, Result};
use std::borrow::Cow;
use std::ffi::{c_int, CStr};
use std::marker::PhantomData;

const MODULE_NAME: &CStr = c"zipfile";

/// Register the "vtablog" module.
pub fn load_module(conn: &Connection) -> Result<()> {
    const MODULE: Module<ZipfileTab> = Module::update_module_with_tx().without_sync();
    let aux: Option<()> = None;
    conn.create_module(MODULE_NAME, &MODULE, aux)
}

/// An instance of the vtablog virtual table
#[repr(C)]
struct ZipfileTab {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab,
    /// Zip file this table accesses
    filename: String,
    /// Host database connection
    db: *mut ffi::sqlite3,
}

#[repr(C)]
struct ZipfileCsr<'vtab> {
    /// Base class. Must be first
    base: ffi::sqlite3_vtab_cursor,
    phantom: PhantomData<&'vtab ZipfileTab>,
}

unsafe impl VTabCursor for ZipfileCsr<'_> {
    fn filter(&mut self, idx_num: c_int, idx_str: Option<&str>, args: &Filters<'_>) -> Result<()> {
        todo!() // zipfileFilter
    }

    fn next(&mut self) -> Result<()> {
        todo!() // zipfileNext
    }

    fn eof(&self) -> bool {
        todo!() // zipfileEof
    }

    fn column(&self, ctx: &mut Context, i: c_int) -> Result<()> {
        todo!() // zipfileColumn
    }

    fn rowid(&self) -> Result<i64> {
        unreachable!() // No row id
    }
}

// 0: Name of file in zip archive
// 1: POSIX mode for file
// 2: Last modification time (secs since 1970)
// 3: Size of object
// 4: Raw data
// 5: Uncompressed data
// 6: Compression method (integer)
// 7: Name of zip file
const ZIPFILE_SCHEMA: &CStr = c"CREATE TABLE y(\
  name PRIMARY KEY,\
  mode,\
  mtime,\
  sz,\
  rawdata,\
  data,\
  method,\
  z HIDDEN\
) WITHOUT ROWID;";
const ZIPFILE_F_COLUMN_IDX: c_int = 7; // Index of column "file" in the above
const ZIPFILE_MX_NAME: c_int = 250; // Windows limitation on filename size

/*
** The buffer should be large enough to contain 3 65536 byte strings - the
** filename, the extra field and the file comment.
*/
const ZIPFILE_BUFFER_SIZE: usize = 200 * 1024;

unsafe impl<'vtab> VTab<'vtab> for ZipfileTab {
    type Aux = ();
    type Cursor = ZipfileCsr<'vtab>;

    fn connect(
        db: &mut VTabConnection,
        aux: Option<&Self::Aux>,
        module_name: &[u8],
        _database_name: &[u8],
        _table_name: &[u8],
        args: &[&[u8]],
    ) -> Result<(Cow<'static, CStr>, Self)> {
        debug_assert_eq!(aux, None);
        debug_assert_eq!(module_name, MODULE_NAME.to_bytes());
        // FIXME filename is optional
        if args.len() != 1 {
            return Err(Error::ModuleError(
                "zipfile constructor requires one argument".to_owned(),
            ));
        }
        let filename = dequote(std::str::from_utf8(args[0])?).to_owned();
        let vtab = Self {
            base: ffi::sqlite3_vtab::default(),
            filename,
            db: unsafe { db.handle() },
        };
        db.config(VTabConfig::DirectOnly)?;
        Ok((Cow::Borrowed(ZIPFILE_SCHEMA), vtab))
    }

    fn best_index(&self, info: &mut IndexInfo) -> Result<bool> {
        let mut unusable = false;
        let mut idx = None;
        for (i, constraint) in info.constraints().enumerate() {
            if constraint.column() != ZIPFILE_F_COLUMN_IDX {
                continue;
            }
            if !constraint.is_usable() {
                unusable = true;
            } else if constraint.operator() == IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ {
                idx = Some(i);
            }
        }
        info.set_estimated_cost(1000.);
        if let Some(i) = idx {
            let mut constraint_usage = info.constraint_usage(i);
            constraint_usage.set_argv_index(1);
            constraint_usage.set_omit(true);
            info.set_idx_num(1);
        } else if unusable {
            return Ok(false);
        }
        Ok(true)
    }

    fn open(&'vtab mut self) -> Result<Self::Cursor> {
        Ok(ZipfileCsr {
            base: ffi::sqlite3_vtab_cursor::default(),
            phantom: PhantomData,
        }) // zipfileOpen
    }
}

impl Drop for ZipfileTab {
    fn drop(&mut self) {
        // zipfileDisconnect
    }
}

impl CreateVTab<'_> for ZipfileTab {
    const KIND: VTabKind = VTabKind::Eponymous;
}

impl UpdateVTab<'_> for ZipfileTab {
    fn delete(&mut self, arg: ValueRef<'_>) -> Result<()> {
        todo!()
    }

    fn insert(&mut self, args: &Inserts<'_>) -> Result<i64> {
        todo!()
    }

    fn update(&mut self, args: &Updates<'_>) -> Result<()> {
        todo!() // zipfileUpdate
    }
}

impl TransactionVTab<'_> for ZipfileTab {
    fn begin(&mut self) -> Result<()> {
        todo!() // zipfileBegin
    }

    fn commit(&mut self) -> Result<()> {
        todo!() // zipfileCommit
    }

    fn rollback(&mut self) -> Result<()> {
        self.commit() // zipfileRollback
    }
}

// zipfileAppendEntry / zipfileSerializeLFH / zipfileNewEntry -> ZipWriter::start_file + ZipWriter::write

// TODO zipfileFindFunction
