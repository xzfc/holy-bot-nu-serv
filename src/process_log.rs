use std::convert;
use std::io;
use std::io::{SeekFrom, BufReader, BufRead, Seek};
use std::fs::File;

pub trait LogProcessor {
    type Error: convert::From<io::Error>;
    fn begin(&mut self) -> Result<Option<u64>, Self::Error>;
    fn commit(&mut self, end_pos: u64) -> Result<(), Self::Error>;
    fn abort(&mut self) -> Result<(), Self::Error>;
    fn process_line(&mut self, line: &String) -> Result<(), Self::Error>;
}

// Contract:
//   abort() is called if any failure happens after succesful begin() and
//   before succesful commit().
//
// In other words, abort() is called when failure happens on lines marked
// with `|` in the following pseudocode:
// {
//     ...
//     begin()
//   | ...
//   | process_line() (zero or more times)
//   | ...
//   | commit()
//     ...
// }
pub fn process_log<T: LogProcessor>(
    fname: &str,
    processor: &mut T) ->
    Result<(), T::Error> {
    // Same as `try!` but also call `processor.abort()` on failure.
    macro_rules! try_abort {
        ($e:expr) => {
            match $e {
                Ok(x) => x,
                e => { processor.abort()?; e? },
            }
        };
    }

    let f = File::open(fname)?;
    let mut f = BufReader::new(&f);

    let mut total_bytes: u64 = 0;
    if let Some(pos) = processor.begin()? {
        eprintln!("process_log: fseek {}", pos);
        total_bytes += pos;
        try_abort!( f.seek(SeekFrom::Start(pos)) );
    }

    let mut lineno = 0;
    loop {
        let mut line = String::new();
        let read_bytes = try_abort!( f.read_line(&mut line) );
        if read_bytes == 0 {
            break;
        }
        if line.ends_with("\n") {
            line.pop();
        }
        total_bytes += read_bytes as u64;
        lineno += 1;
        try_abort!( processor.process_line(&line) );
        if lineno % 1000 == 0 {
            eprintln!("process_log: line {}", lineno);
            try_abort!( processor.commit(total_bytes) );
            let ensure_total_bytes = processor.begin()?;
            if ensure_total_bytes != Some(total_bytes) {
                eprintln!("process_log: error ensure_total_bytes != total_bytes");
                return Ok(()) // TODO: return error
            }
        }
    }

    try_abort!( processor.commit(total_bytes) );

    Ok(())
}
