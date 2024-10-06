pub const MAX_EXTENSIONS: u32 = 16;

pub const fn extensions(in_check: bool, extension_count: u32) -> u32 {
    let mut extensions = 0;

    if extension_count < MAX_EXTENSIONS {
        if in_check {
            extensions = 1;
        }
    }

    extensions
}
