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
    Tag(String),
    Collect(usize, usize),
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
            Tag(tag) => write!(f, "HTML element with tag `{}` isn't supported", tag),
            Collect(running, total) => write!(f, "Traversed {} assets, but collected {}", running, total),
        }
    }
}

#[derive(Debug)]
pub struct AssetError {
    inner:   InnerError,
    details: Option<String>,
}

impl AssetError {
    pub(crate) fn bad_tag<S: AsRef<str>>(tag: S) -> Self {
        InnerError::Tag(tag.as_ref().to_string()).into()
    }

    pub(crate) fn bad_collect(running: usize, total: usize) -> Self {
        InnerError::Collect(running, total).into()
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
