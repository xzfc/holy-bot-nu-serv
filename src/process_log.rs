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

    let mut f = File::open(fname)?;

    if let Some(pos) = processor.begin()? {
        println!("fseek {}", pos);
        try_abort!( f.seek(SeekFrom::Start(pos)) );
    }

    let mut n = 0;
    for line in BufReader::new(&f).lines() {
        let line = try_abort!(line);
        n += 1;
        if n % 1000 == 0 {
            println!("Line {}", n);
        }
        try_abort!( processor.process_line(&line) );
    }

    let current = try_abort!( f.seek(SeekFrom::Current(0)) );

    try_abort!( processor.commit(current) );

    Ok(())
}
