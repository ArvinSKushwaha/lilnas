#[macro_export]
macro_rules! try_print {
    ($($printables:expr),+$(,)?) => {
        write!(std::io::stdout(), $($printables),*)
    };
}

#[macro_export]
macro_rules! try_println {
    ($($printables:expr),+$(,)?) => {
        writeln!(std::io::stdout(), $($printables),*)
    };
}

#[macro_export]
macro_rules! try_eprint {
    ($($printables:expr),+$(,)?) => {
        write!(std::io::stderr(), $($printables),*)
    };
}

#[macro_export]
macro_rules! try_eprintln {
    ($($printables:expr),+$(,)?) => {
        writeln!(std::io::stderr(), $($printables),*)
    };
}
