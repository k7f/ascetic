use pixels::wgpu;
use ascetic_vis::piet;

#[derive(Debug)]
pub enum Error {
    Fatal(Box<dyn std::error::Error>),
    PietFailure(piet::Error),
    WinitFailure(winit::error::OsError),
    SwapChainFailure(wgpu::SwapChainError),
    PixelsFailure(pixels::Error),
    MissingPixmap,
    PixmapRenderingFailure,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;

        match self {
            Fatal(err) => err.fmt(f),
            PietFailure(err) => err.fmt(f),
            WinitFailure(err) => err.fmt(f),
            SwapChainFailure(err) => err.fmt(f),
            PixelsFailure(err) => err.fmt(f),
            MissingPixmap => write!(f, "Missing pixmap"),
            PixmapRenderingFailure => write!(f, "Pixmap rendering failed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::Fatal(err)
    }
}

impl From<piet::Error> for Error {
    fn from(err: piet::Error) -> Self {
        Error::PietFailure(err)
    }
}

impl From<winit::error::OsError> for Error {
    fn from(err: winit::error::OsError) -> Self {
        Error::WinitFailure(err)
    }
}

impl From<pixels::Error> for Error {
    fn from(err: pixels::Error) -> Self {
        match err {
            pixels::Error::Swapchain(wgpu::SwapChainError::OutOfMemory) => Error::Fatal(err.into()),
            pixels::Error::Swapchain(err) => Error::SwapChainFailure(err),
            _ => Error::PixelsFailure(err),
        }
    }
}
