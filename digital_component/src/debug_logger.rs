use std::fmt::Arguments;

pub struct DebugLogger {
    current_depth: i32,
    log_depth: i32,
}

static mut LOGGER: DebugLogger = DebugLogger {
    current_depth: 0,
    log_depth: 1,
};

pub fn begin_context() {
    unsafe {
        LOGGER.current_depth += 1;
    }
}

pub fn log(args: Arguments) {
    unsafe {
        if LOGGER.log_depth >= LOGGER.current_depth {
            println!("{}", args);
        }
    }
}

pub fn end_context() {
    unsafe {
        LOGGER.current_depth -= 1;
    }
}
