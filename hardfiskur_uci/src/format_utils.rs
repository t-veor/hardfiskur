use std::fmt::Display;

pub struct SpaceSepFormatter<'a, 'b> {
    formatter: &'b mut std::fmt::Formatter<'a>,
    written: bool,
}

impl<'a, 'b> SpaceSepFormatter<'a, 'b> {
    pub fn new(f: &'b mut std::fmt::Formatter<'a>) -> Self {
        Self {
            formatter: f,
            written: false,
        }
    }

    fn sep(&mut self) -> std::fmt::Result {
        if !self.written {
            self.written = true;
        } else {
            write!(self.formatter, " ")?;
        }

        Ok(())
    }

    pub fn push<T: Display>(&mut self, value: &T) -> std::fmt::Result {
        self.sep()?;
        write!(self.formatter, "{value}")
    }

    pub fn push_str(&mut self, name: &str) -> std::fmt::Result {
        self.sep()?;
        write!(self.formatter, "{name}")
    }

    pub fn push_pair<T: Display>(&mut self, name: &str, value: &T) -> std::fmt::Result {
        self.sep()?;

        write!(self.formatter, "{name} {value}")?;

        Ok(())
    }

    pub fn push_option<T: Display + Sized>(
        &mut self,
        name: &str,
        value: Option<T>,
    ) -> std::fmt::Result {
        if let Some(value) = value {
            self.sep()?;

            write!(self.formatter, "{name} {value}")?;
        }

        Ok(())
    }

    pub fn push_option_ref<T: Display>(
        &mut self,
        name: &str,
        value: Option<&T>,
    ) -> std::fmt::Result {
        if let Some(value) = value {
            self.push_pair(name, value);
        }

        Ok(())
    }
}
