
use std::intrinsics::type_name;

#[inline]
pub fn get_type_name<T: 'static>() -> String {
    unsafe { type_name::<T>().to_owned().replace("::", ".") }
}
