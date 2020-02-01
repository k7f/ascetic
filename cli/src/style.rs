use std::fmt;

pub struct WithStyle<S: AsRef<str>> {
    message: S,
    color:   &'static str,
    style:   &'static str,
}

impl<S: AsRef<str>> From<S> for WithStyle<S> {
    fn from(message: S) -> Self {
        WithStyle {
            message,
            color: "49",
            style: "",
        }
    }
}

impl<S: AsRef<str>> fmt::Display for WithStyle<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[{}{}m{}\x1B[0m", self.style, self.color, self.message.as_ref())
    }
}

pub trait Styled<S>: Into<WithStyle<S>> where S: AsRef<str> {
    fn bright_red(self) -> WithStyle<S> {
        let mut this = self.into();

        this.color = "49;91";
        this
    }

    fn bright_green(self) -> WithStyle<S> {
        let mut this = self.into();

        this.color = "49;92";
        this
    }

    fn bright_yellow(self) -> WithStyle<S> {
        let mut this = self.into();

        this.color = "49;93";
        this
    }

    fn bold(self) -> WithStyle<S> {
        let mut this = self.into();

        this.style = "1;";
        this
    }
}

impl<S, T> Styled<S> for T where S: AsRef<str>, T: Into<WithStyle<S>> {}
