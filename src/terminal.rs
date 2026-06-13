

// TODO: add stylings: bold, colours etc.
// TODO: have a token representation for precise error location esp when giving feedback
// above only feasible when token stream is high e.g via addition of more cliu args.

pub mod mig_debug {
    #[cfg(debug_assertions)]
    #[macro_export]
    macro_rules! debug_print {
        ($($arg:tt)*) => {
            println!("[ DEBUG ] :: {}", format_args!($($arg)*));
        };
    }

    #[macro_export]
    macro_rules! print_message {
        (info, $($arg:tt)*) => {
            println!("[ INFO ] :: {}", format_args!($($arg)*));
        };

        (warn, $($arg:tt)*) => {
            println!("[ WARN ] :: {}", format_args!($($arg)*));
        };

        (error, $($arg:tt)*) => {
            println!("[ ERROR ] :: {}", format_args!($($arg)*));
        };
    }

}


