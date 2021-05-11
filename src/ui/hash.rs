#[macro_export]
macro_rules! hash {
    ($s:expr) => {{
        use ahash::RandomState;
        use std::hash::{Hash, Hasher, BuildHasher};

        static mut STATE: Option<RandomState> = None;

        let id = $s;

        let state = unsafe { STATE.get_or_insert(RandomState::new()) };
        let mut s = state.build_hasher();
        id.hash(&mut s);
        s.finish()
    }};
    () => {{
        let id = concat!(file!(), line!(), column!());
        hash!(id)
    }};
    ($($s:expr),*) => {{
        let mut s: u128 = 0;
        $(s += $crate::hash!($s) as u128;)*
        $crate::hash!(s)
    }};
}
