use crate::parameters::MAX_EXTENSIONS;

pub const fn extensions(in_check: bool, extension_count: i16) -> i16 {
    let mut extensions = 0;

    if extension_count < MAX_EXTENSIONS {
        if in_check {
            extensions = 1;
        }
    }

    extensions
}
