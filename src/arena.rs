use core::marker::PhantomData;
use core::slice;

#[derive(Clone, Copy, Debug)]
pub struct RawSpan {
    pub ptr: *const u8,
    pub len: usize,
}

impl RawSpan {
    pub fn empty() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ptr.is_null() || self.len == 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RawSlice<T> {
    pub ptr: *const T,
    pub len: usize,
    _marker: PhantomData<T>,
}

impl<T> RawSlice<T> {
    pub fn empty() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ptr.is_null() || self.len == 0
    }
}

pub fn capture_bytes(slice: &[u8]) -> RawSpan {
    RawSpan {
        ptr: slice.as_ptr(),
        len: slice.len(),
    }
}

pub fn capture_slice<T>(slice: &[T]) -> RawSlice<T> {
    RawSlice {
        ptr: slice.as_ptr(),
        len: slice.len(),
        _marker: PhantomData,
    }
}

pub unsafe fn span_to_vec(span: RawSpan) -> Vec<u8> {
    if span.is_empty() {
        return Vec::new();
    }
    slice::from_raw_parts(span.ptr, span.len).to_vec()
}

pub unsafe fn span_to_string(span: RawSpan) -> String {
    String::from_utf8_lossy(&span_to_vec(span)).into_owned()
}

pub unsafe fn span_score(span: RawSpan) -> u64 {
    if span.is_empty() {
        return 0;
    }
    let mut acc = 0xfeed_beefu64;
    for &b in slice::from_raw_parts(span.ptr, span.len) {
        acc = acc.rotate_left(5) ^ b as u64;
    }
    acc
}

pub unsafe fn read_at<T: Copy>(raw: RawSlice<T>, index: usize) -> T {
    *raw.ptr.add(index)
}

pub unsafe fn slice_prefix_sum_i16(raw: RawSlice<i16>, extra: usize) -> i64 {
    if raw.is_empty() {
        return 0;
    }
    let count = raw.len.saturating_add(extra);
    let view = slice::from_raw_parts(raw.ptr, count);
    view.iter().fold(0i64, |acc, &v| acc + v as i64)
}
