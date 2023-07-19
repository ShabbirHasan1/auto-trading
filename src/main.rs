mod local_bourse;

use auto_trading::*;

lazy_static::lazy_static! {
    static ref CLIENT: reqwest::Client
        = reqwest::ClientBuilder::new().timeout(std::time::Duration::from_secs(5)).build().unwrap();
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};

    ($name:expr) => {
        format!("{}::{}", function!(), $name)
    };
}

struct MyFn {
    name: String,
    inner: Box<dyn Fn()>,
}

impl MyFn {
    fn new(f: impl Fn() + 'static) -> Self {
        Self {
            name: format!("{}:{}:{}: {}", file!(), line!(), column!(), function!()),
            inner: Box::new(f),
        }
    }
}

struct St {
    inner: Vec<MyFn>,
}

impl St {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }

    // #[force_inline]
    fn add(&mut self, f: impl Into<MyFn>) {}
}

// #[tokio::main]
fn main() {
    let mut st = St::new();

    st.add(MyFn::new(|| {
        println!("aa");
    }));

    st.add(MyFn::new(|| {
        println!("bb");
    }));

    st.inner.iter().for_each(|v| println!("{}", v.name));
}
