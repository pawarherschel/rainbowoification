#[macro_export]
macro_rules! time_it {
    ($comment:literal => $stmt:stmt) => {{
        time_it!(concat!($comment, "") => {$stmt})
    }};
    (at once | $comment:literal => $stmt:stmt) => {{
        time_it!(at once | concat!($comment, "") => {$stmt})
    }};
    ($comment:expr => $stmt:stmt) => {{
        #[allow(unused_imports)]
        use std::io::Write as _;
        print!("{}", $comment);
        let _ = std::io::stdout().flush();
        let start = std::time::Instant::now();
        let result = { $stmt };
        let duration = start.elapsed();
        println!(" => {:?}", duration);
        result
    }};
    (at once | $comment:expr => $stmt:stmt) => {{
        #[allow(unused_imports)]
        use std::io::Write as _;
        let start = std::time::Instant::now();
        let result = { $stmt };
        let duration = start.elapsed();
        println!("{} => {:?}", $comment, duration);
        result
    }};
}
#[macro_export]
macro_rules! debug {
    ($val:expr) => {
        #[cfg(debug_assertions)]
        {
            dbg!($val)
        }
    };
    ($($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        {
            dbg!($($val),+)
        }
    };
}

#[macro_export]
macro_rules! lerp {
    ($start: expr, $t: expr, $end: expr) => {{
        (1_f64 - $t) * $start + $t * $end
    }};
}

#[macro_export]
macro_rules! remap {
    (value: $value: expr, from: $min1: expr, $max1: expr, to: $min2: expr, $max2: expr) => {{
        let value = $value;
        let min1 = $min1;
        let max1 = $max1;
        let min2 = $min2;
        let max2 = $max2;

        min2 + (value - min1) * (max2 - min2) / (max1 - min1)
    }};
}
