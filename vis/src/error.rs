use crate::{Crumb, CrumbId, GroupId};

pub(crate) trait DetailedError {
    fn with_string(self, details: String) -> VisError;
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
    CrumbMismatch(String, Crumb, CrumbId),
    CrumbMissingForId(CrumbId),
    GroupMissingForId(GroupId),
    LayerMissingForId(GroupId),
    GroupReuseAttempt(GroupId),
    CrumbsOfAGroupOverflow(GroupId, usize),
    GroupsOfAGroupOverflow(GroupId, usize),
    BuilderOverflow(String, usize),
}

macro_rules! impl_inner_error {
    ($error:ty, $variant:ident) => {
        impl DetailedError for $error {
            fn with_string(self, details: String) -> VisError {
                InnerError::from(self).with_string(details)
            }
        }

        impl From<$error> for InnerError {
            #[inline]
            fn from(err: $error) -> Self {
                InnerError::$variant(err)
            }
        }

        impl From<$error> for VisError {
            #[inline]
            fn from(err: $error) -> Self {
                VisError { inner: err.into(), details: None }
            }
        }
    };
}

impl_inner_error!(std::io::Error, IO);

impl DetailedError for InnerError {
    #[inline]
    fn with_string(self, details: String) -> VisError {
        VisError::from(self).with_string(details)
    }
}

impl std::fmt::Display for InnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use InnerError::*;

        match self {
            IO(err) => write!(f, "IO error {:?}", err),
            CrumbMismatch(name, crumb, crumb_id) => {
                write!(f, "Unexpected {:?} instead of {} for {:?}", crumb, name, crumb_id)
            }
            CrumbMissingForId(crumb_id) => write!(f, "Crumb missing for {:?}", crumb_id),
            GroupMissingForId(group_id) => write!(f, "Group missing for {:?}", group_id),
            LayerMissingForId(group_id) => write!(f, "Layer missing for {:?}", group_id),
            GroupReuseAttempt(group_id) => write!(f, "Reuse attempt for {:?}", group_id),
            CrumbsOfAGroupOverflow(group_id, index) => {
                write!(f, "Index {} overflows grouped crumbs for {:?}", index, group_id)
            }
            GroupsOfAGroupOverflow(group_id, index) => {
                write!(f, "Index {} overflows grouped groups for {:?}", index, group_id)
            }
            BuilderOverflow(name, num_items) => write!(
                f,
                "Attempt to override declared number of {} {} in a builder",
                num_items, name
            ),
        }
    }
}

#[derive(Debug)]
pub struct VisError {
    inner:   InnerError,
    details: Option<String>,
}

impl VisError {
    pub(crate) fn crumb_mismatch<S: AsRef<str>>(name: S, crumb: Crumb, crumb_id: CrumbId) -> Self {
        InnerError::CrumbMismatch(name.as_ref().to_string(), crumb, crumb_id).into()
    }

    pub(crate) fn crumb_missing_for_id(crumb_id: CrumbId) -> Self {
        InnerError::CrumbMissingForId(crumb_id).into()
    }

    pub(crate) fn group_missing_for_id(group_id: GroupId) -> Self {
        InnerError::GroupMissingForId(group_id).into()
    }

    pub(crate) fn layer_missing_for_id(group_id: GroupId) -> Self {
        InnerError::LayerMissingForId(group_id).into()
    }

    pub(crate) fn group_reuse_attempt(group_id: GroupId) -> Self {
        InnerError::GroupReuseAttempt(group_id).into()
    }

    pub(crate) fn crumbs_of_a_group_overflow(group_id: GroupId, index: usize) -> Self {
        InnerError::CrumbsOfAGroupOverflow(group_id, index).into()
    }

    pub(crate) fn groups_of_a_group_overflow(group_id: GroupId, index: usize) -> Self {
        InnerError::GroupsOfAGroupOverflow(group_id, index).into()
    }

    pub(crate) fn builder_overflow<S: AsRef<str>>(name: S, num_items: usize) -> Self {
        InnerError::BuilderOverflow(name.as_ref().to_string(), num_items).into()
    }

    pub(crate) fn std_io<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        std::io::Error::new(std::io::ErrorKind::Other, err).into()
    }
}

impl DetailedError for VisError {
    fn with_string(mut self, details: String) -> VisError {
        self.details = Some(details);
        self
    }
}

impl From<InnerError> for VisError {
    #[inline]
    fn from(inner: InnerError) -> Self {
        VisError { inner, details: None }
    }
}

impl std::fmt::Display for VisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref details) = self.details {
            write!(f, "{:?}: {}", self.inner, details)
        } else {
            self.inner.fmt(f)
        }
    }
}

impl std::error::Error for VisError {}
