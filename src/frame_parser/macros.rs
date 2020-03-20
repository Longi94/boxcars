macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    }
}
