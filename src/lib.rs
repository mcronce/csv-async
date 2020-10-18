#![deny(missing_docs)]

/*!
The `csv-async` crate provides a fast and flexible CSV reader and writer, 
which is intended to be run in asynchronous environment - i.e.
inside functions with `async` attribute called by tasks run by executor.
This library does not imply using any particular executor (is executor agnostic).
Unit tests and documentation snippets uses `async-std` crate.
Synchronous interface for reading and writing CSV files is not contained in this crate,
please use `csv` crate for this. This crate attempts to closely mimic `csv` crate API.

TODO: The [tutorial](tutorial/index.html) is a good place to start if you're new to
Rust.

TODO: The [cookbook](cookbook/index.html) will give you a variety of complete Rust
programs that do CSV reading and writing.

# Brief overview

The primary types in this crate are
[`AsyncReader`](struct.AsyncReader.html)
and
[`AsyncWriter`](struct.AsyncWriter.html),
for reading and writing CSV data respectively.
Correspondingly, to support CSV data with custom field or record delimiters
(among many other things), you should use either a
[`AsyncReaderBuilder`](struct.AsyncReaderBuilder.html)
or a
[`AsyncWriterBuilder`](struct.AsyncWriterBuilder.html),
depending on whether you're reading or writing CSV data.

The standard CSV record types are
[`StringRecord`](struct.StringRecord.html)
and
[`ByteRecord`](struct.ByteRecord.html).
`StringRecord` should be used when you know your data to be valid UTF-8.
For data that may be invalid UTF-8, `ByteRecord` is suitable.

Finally, the set of errors is described by the
[`Error`](struct.Error.html)
type.

The rest of the types in this crate mostly correspond to more detailed errors,
position information, configuration knobs or iterator types.

# Setup

Add this to your `Cargo.toml`:

```toml
[dependencies]
csv-async = "0.0.3"
```

# Example

This example shows how to read and write CSV file in asynchronous context and get into some record details.

Sample input file:
```csv
city,region,country,population
Southborough,MA,United States,9686
Northbridge,MA,United States,14061
Marlborough,MA,United States,38334
Springfield,MA,United States,152227
Springfield,MO,United States,150443
Springfield,NJ,United States,14976
Concord,NH,United States,42605
```

```no_run
use std::error::Error;
use std::process;
use futures::stream::StreamExt;
use async_std::fs::File;

async fn filter_by_region(region:&str, file_in:&str, file_out:&str) -> Result<(), Box<dyn Error>> {
    // Function reads CSV file that has column named "region"
    // at second position (index = 1).
    // It writes to new file only rows with region equal to passed argument
    // and remove region column.
    let mut rdr = csv_async::AsyncReader::from_reader(
        File::open(file_in).await?
    );
    let mut wri = csv_async::AsyncWriter::from_writer(
        File::create(file_out).await?
    );
    wri.write_record(rdr
        .headers()
        .await?.into_iter()
        .filter(|h| *h != "region")
    ).await?;
    let mut records = rdr.records();
    while let Some(record) = records.next().await {
        let record = record?;
        match record.get(1) {
            Some(reg) if reg == region => 
                wri.write_record(record
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != 1)
                    .map(|(_, s)| s)
                ).await?,
            _ => {},
        }
    }
    Ok(())
}

fn main() {
    async_std::task::block_on(async {
        if let Err(err) = filter_by_region(
            "MA",
            "/tmp/all_regions.csv",
            "/tmp/MA_only.csv"
        ).await {
            println!("error running filter_by_region: {}", err);
            process::exit(1);
        }
    });
}
```

TODO: There are more examples in the [cookbook](cookbook/index.html).
*/

#[cfg(test)]
mod tests {
    use std::error::Error;
    use futures::stream::StreamExt;
    use async_std::fs::File;
   
    async fn crete_async(file:&str) -> Result<(), Box<dyn Error>> {
        // Build the CSV reader and iterate over each record.
        let mut wri = crate::AsyncWriter::from_writer(
            File::create(file).await?
        );
        wri.write_record(&["city","region","country","population"]).await?;
        wri.write_record(&["Northbridge","MA","United States","14061"]).await?;
        wri.write_record(&["Westborough","MA","United States","29313"]).await?;
        wri.write_record(&["Springfield","NJ","United States","14976"]).await?;
        wri.flush().await?;
        Ok(())
    }
   
    async fn copy_async(file_in:&str, file_out:&str) -> Result<(), Box<dyn Error>> {
        let mut rdr = crate::AsyncReader::from_reader(
            File::open(file_in).await?
        );
        let mut wri = crate::AsyncWriter::from_writer(
            File::create(file_out).await?
        );
        wri.write_record(rdr.headers().await?.into_iter()).await?;
        let mut records = rdr.records();
        while let Some(record) = records.next().await {
            wri.write_record(&record?).await?;
        }
        Ok(())
    }
   
    #[test]
    fn test_on_files() {
        use std::io::Read;
        use std::hash::Hasher;
        std::fs::create_dir_all("examples/data").unwrap();
        let file_in  = "examples/data/smallpop.csv";
        let file_out = "examples/data/smallpop_out.csv";

        async_std::task::block_on(async {
            if let Err(err) = crete_async(file_in).await {
                assert!(false, "error running crete_async: {}", err);
            }
            if let Err(err) = copy_async(file_in, file_out).await {
                assert!(false, "error running copy_async: {}", err);
            }
        });
        
        let mut bytes_in  = vec![];
        std::fs::File::open(file_in).unwrap().read_to_end(&mut bytes_in).unwrap();
        let mut hasher_in = std::collections::hash_map::DefaultHasher::new();
        hasher_in.write(&bytes_in);

        let mut bytes_out = vec![];
        std::fs::File::open(file_out).unwrap().read_to_end(&mut bytes_out).unwrap();
        let mut hasher_out = std::collections::hash_map::DefaultHasher::new();
        hasher_out.write(&bytes_out);

        assert_eq!(hasher_in.finish(), hasher_out.finish(), "Cloned file {} is different than source {}", file_out, file_in);
        
        std::fs::remove_file(file_in).unwrap();
        std::fs::remove_file(file_out).unwrap();
    }
}


pub use crate::byte_record::{ByteRecord, ByteRecordIter, Position};
pub use crate::error::{
    Error, ErrorKind, FromUtf8Error, IntoInnerError, Result, Utf8Error,
};
pub use crate::string_record::{StringRecord, StringRecordIter};
pub use crate::async_reader::{
    AsyncReader, AsyncReaderBuilder, ByteRecordsIntoStream, ByteRecordsStream,
    StringRecordsIntoStream, StringRecordsStream,
};
pub use crate::async_writer::{AsyncWriter, AsyncWriterBuilder};

mod byte_record;
mod error;
mod string_record;
mod async_reader;
mod async_writer;

// pub mod cookbook;
// pub mod tutorial;

/// The quoting style to use when writing CSV data.
#[derive(Clone, Copy, Debug)]
pub enum QuoteStyle {
    /// This puts quotes around every field. Always.
    Always,
    /// This puts quotes around fields only when necessary.
    ///
    /// They are necessary when fields contain a quote, delimiter or record
    /// terminator. Quotes are also necessary when writing an empty record
    /// (which is indistinguishable from a record with one empty field).
    ///
    /// This is the default.
    Necessary,
    /// This puts quotes around all fields that are non-numeric. Namely, when
    /// writing a field that does not parse as a valid float or integer, then
    /// quotes will be used even if they aren't strictly necessary.
    NonNumeric,
    /// This *never* writes quotes, even if it would produce invalid CSV data.
    Never,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl QuoteStyle {
    #[allow(dead_code)]
    fn to_core(self) -> csv_core::QuoteStyle {
        match self {
            QuoteStyle::Always => csv_core::QuoteStyle::Always,
            QuoteStyle::Necessary => csv_core::QuoteStyle::Necessary,
            QuoteStyle::NonNumeric => csv_core::QuoteStyle::NonNumeric,
            QuoteStyle::Never => csv_core::QuoteStyle::Never,
            _ => unreachable!(),
        }
    }
}

impl Default for QuoteStyle {
    fn default() -> QuoteStyle {
        QuoteStyle::Necessary
    }
}

/// A record terminator.
///
/// Use this to specify the record terminator while parsing CSV. The default is
/// CRLF, which treats `\r`, `\n` or `\r\n` as a single record terminator.
#[derive(Clone, Copy, Debug)]
pub enum Terminator {
    /// Parses `\r`, `\n` or `\r\n` as a single record terminator.
    CRLF,
    /// Parses the byte given as a record terminator.
    Any(u8),
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Terminator {
    /// Convert this to the csv_core type of the same name.
    fn to_core(self) -> csv_core::Terminator {
        match self {
            Terminator::CRLF => csv_core::Terminator::CRLF,
            Terminator::Any(b) => csv_core::Terminator::Any(b),
            _ => unreachable!(),
        }
    }
}

impl Default for Terminator {
    fn default() -> Terminator {
        Terminator::CRLF
    }
}

/// The whitespace preservation behavior when reading CSV data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trim {
    /// Preserves fields and headers. This is the default.
    None,
    /// Trim whitespace from headers.
    Headers,
    /// Trim whitespace from fields, but not headers.
    Fields,
    /// Trim whitespace from fields and headers.
    All,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Trim {
    fn should_trim_fields(&self) -> bool {
        self == &Trim::Fields || self == &Trim::All
    }

    fn should_trim_headers(&self) -> bool {
        self == &Trim::Headers || self == &Trim::All
    }
}

impl Default for Trim {
    fn default() -> Trim {
        Trim::None
    }
}
