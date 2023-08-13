use std::fmt;

pub fn debug_format_option_display<T>(this: &Option<T>, f: &mut fmt::Formatter) -> fmt::Result
where
    T: fmt::Display,
{
    if let Some(url) = this {
        write!(f, "Some({})", url)
    } else {
        write!(f, "None")
    }
}

pub fn debug_format_vec_display<T>(this: &Vec<T>, f: &mut fmt::Formatter) -> fmt::Result
where
    T: fmt::Display,
{
    write!(f, "[")?;
    for url in this {
        write!(f, "{}, ", url)?;
    }
    write!(f, "]")
}
