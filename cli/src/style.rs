use std::fmt;

pub struct WithStyle<S: AsRef<str>> {
    message: S,
    color:   &'static str,
    style:   &'static str,
}

impl<S: AsRef<str>> From<S> for WithStyle<S> {
    fn from(message: S) -> Self {
        WithStyle { message, color: "49", style: "" }
    }
}

impl<S: AsRef<str>> fmt::Display for WithStyle<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[{}{}m{}\x1B[0m", self.style, self.color, self.message.as_ref())
    }
}

macro_rules! define_color_method {
    ($name:ident,$code:expr) => {
        fn $name(self) -> WithStyle<S> {
            let mut this = self.into();

            this.color = concat!("49;", $code);
            this
        }
    };
}

pub trait Styled<S>: Into<WithStyle<S>>
where
    S: AsRef<str>,
{
    define_color_method!(red, 31);
    define_color_method!(green, 32);
    define_color_method!(yellow, 33);
    define_color_method!(blue, 34);
    define_color_method!(magenta, 35);
    define_color_method!(cyan, 36);
    define_color_method!(white, 37);
    define_color_method!(bright_red, 91);
    define_color_method!(bright_green, 92);
    define_color_method!(bright_yellow, 93);
    define_color_method!(bright_blue, 94);
    define_color_method!(bright_magenta, 95);
    define_color_method!(bright_cyan, 96);
    define_color_method!(bright_white, 97);

    fn bold(self) -> WithStyle<S> {
        let mut this = self.into();

        this.style = "1;";
        this
    }
}

impl<S, T> Styled<S> for T
where
    S: AsRef<str>,
    T: Into<WithStyle<S>>,
{
}
