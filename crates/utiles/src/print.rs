//! `safe_print!` and `safe_println!` macros
//!
//! REF: <https://github.com/rust-lang/rust/blob/master/compiler/rustc_driver_impl/src/print.rs>
use std::fmt;
use std::io::{self, Write as _};

#[macro_export]
macro_rules! safe_print {
    ($($arg:tt)*) => {{
        #[allow(clippy::unwrap_used)]
        $crate::print::print(std::format_args!($($arg)*)) .unwrap();
    }};
}

#[macro_export]
macro_rules! safe_println {
    ($($arg:tt)*) => {
        safe_print!("{}\n", std::format_args!($($arg)*))
    };
}

pub fn print(args: fmt::Arguments<'_>) -> Result<(), io::Error> {
    if let Err(err) = io::stdout().write_fmt(args) {
        if err.kind() == io::ErrorKind::BrokenPipe {
            // This is a common error when the output is piped to a command that exits
            // before this process finishes writing. We can ignore it.
            Ok(())
        } else if err.kind() == io::ErrorKind::WriteZero {
            // This error indicates that the write buffer is full and we can't write to it.
            // We can ignore it as well.
            Ok(())
        } else {
            // For any other error, we should panic.
            Err(io::Error::new(
                err.kind(),
                format!("Error writing to stdout: {err}"),
            ))
        }
    } else {
        // Flush the output to ensure it is written immediately.
        io::stdout().flush()?;
        Ok(())
    }
}
