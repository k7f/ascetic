pub(crate) trait DetailedError {
    fn with_string(self, details: String) -> AssetError;
}

#[macro_export]
macro_rules! detailed_error {
    ($($arg:tt)*) => {{
        |err| {
            let message = format!($($arg)*);
            error!("{}", message);
            err.with_string(message)
        }
    }}
}

#[derive(Debug)]
enum InnerError {
    IO(std::io::Error),
    TT(tinytemplate::error::Error),
    FlagClash(String),
    TagClash(String),
    TagUnrenderable(String),
    MismatchedCollect(usize, usize),
    TemplateTargetClash(String),
}

macro_rules! impl_inner_error {
    ($error:ty, $variant:ident) => {
        impl DetailedError for $error {
            fn with_string(self, details: String) -> AssetError {
                InnerError::from(self).with_string(details)
            }
        }

        impl From<$error> for InnerError {
            #[inline]
            fn from(err: $error) -> Self {
                InnerError::$variant(err)
            }
        }

        impl From<$error> for AssetError {
            #[inline]
            fn from(err: $error) -> Self {
                AssetError { inner: err.into(), details: None }
            }
        }
    };
}

impl_inner_error!(std::io::Error, IO);
impl_inner_error!(tinytemplate::error::Error, TT);

impl DetailedError for InnerError {
    #[inline]
    fn with_string(self, details: String) -> AssetError {
        AssetError::from(self).with_string(details)
    }
}

impl std::fmt::Display for InnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use InnerError::*;

        match self {
            IO(err) => write!(f, "IO error {:?}", err),
            TT(err) => write!(f, "TT error {:?}", err),
            FlagClash(flag) => {
                write!(f, "Multiple copies of flag `{}` can't belong to a single asset", flag)
            }
            TagClash(tag) => write!(
                f,
                "Multiple HTML elements with tag `{}` can't be rendered from a single asset",
                tag
            ),
            TagUnrenderable(tag) => write!(f, "HTML element with tag `{}` can't be rendered", tag),
            MismatchedCollect(running, total) => {
                write!(f, "Traversed {} assets, but collected {}", running, total)
            }
            TemplateTargetClash(template_target) => {
                write!(f, "Duplicated template target path {:?}", template_target)
            }
        }
    }
}

#[derive(Debug)]
pub struct AssetError {
    inner:   InnerError,
    details: Option<String>,
}

impl AssetError {
    pub(crate) fn flag_clash<S: AsRef<str>>(flag: S) -> Self {
        InnerError::FlagClash(flag.as_ref().to_string()).into()
    }

    pub(crate) fn tag_clash<S: AsRef<str>>(tag: S) -> Self {
        InnerError::TagClash(tag.as_ref().to_string()).into()
    }

    pub(crate) fn tag_unrenderable<S: AsRef<str>>(tag: S) -> Self {
        InnerError::TagUnrenderable(tag.as_ref().to_string()).into()
    }

    pub(crate) fn mismatched_collect(running: usize, total: usize) -> Self {
        InnerError::MismatchedCollect(running, total).into()
    }

    pub(crate) fn template_target_clash<S: AsRef<str>>(template_target: S) -> Self {
        InnerError::TemplateTargetClash(template_target.as_ref().to_string()).into()
    }

    pub(crate) fn std_io<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        std::io::Error::new(std::io::ErrorKind::Other, err).into()
    }
}

impl DetailedError for AssetError {
    fn with_string(mut self, details: String) -> AssetError {
        self.details = Some(details);
        self
    }
}

impl From<InnerError> for AssetError {
    #[inline]
    fn from(inner: InnerError) -> Self {
        AssetError { inner, details: None }
    }
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref details) = self.details {
            write!(f, "{:?}: {}", self.inner, details)
        } else {
            self.inner.fmt(f)
        }
    }
}

impl std::error::Error for AssetError {}
