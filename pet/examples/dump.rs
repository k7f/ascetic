#[macro_use]
extern crate log;

use std::{path::PathBuf, fmt, error::Error};
use ascetic_pet::{PNML, Net, Page};

#[derive(Debug)]
enum DumpError {
    PlaceNotFound(String, String),
    TransitionNotFound(String, String),
    PageNotFound(String, String),
    NetNotFound(String, String),
}

impl fmt::Display for DumpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DumpError::*;

        match self {
            PlaceNotFound(place_id, page_id) => {
                write!(f, "Place \"{}\" not found on page \"{}\".", place_id, page_id)
            }
            TransitionNotFound(transition_id, page_id) => {
                write!(f, "Transition \"{}\" not found on page \"{}\".", transition_id, page_id)
            }
            PageNotFound(page_id, net_id) => {
                write!(f, "Page \"{}\" not found in net \"{}\".", page_id, net_id)
            }
            NetNotFound(net_id, path) => {
                write!(f, "Net \"{}\" not found in PNML file \"{}\".", net_id, path)
            }
        }
    }
}

impl Error for DumpError {}

struct AppLogger(log::Level);

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static mut APP_LOGGER: AppLogger = AppLogger(log::Level::Warn);

#[derive(Debug)]
struct App {
    path:          PathBuf,
    net_id:        Option<String>,
    page_id:       Option<String>,
    place_id:      Option<String>,
    transition_id: Option<String>,
    verbosity:     u32,
}

impl App {
    const DEFAULT_PATH: &'static str = "data/pnml/test.pnml";

    fn new() -> Result<Self, Box<dyn Error>> {
        let mut path = None;
        let mut net_id = None;
        let mut page_id = None;
        let mut place_id = None;
        let mut transition_id = None;
        let mut verbosity = 0;

        for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
            match next_arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                "-vvv" => verbosity += 3,
                "--net" | "--page" | "--place" | "--transition" => {}
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        match prev_arg.as_str() {
                            "--net" => net_id = Some(next_arg),
                            "--page" => page_id = Some(next_arg),
                            "--place" => place_id = Some(next_arg),
                            "--transition" => transition_id = Some(next_arg),
                            _ => path = Some(PathBuf::from(arg)),
                        }
                    }
                }
            }
        }

        let path = path.unwrap_or_else(|| Self::DEFAULT_PATH.into());

        let log_level = match verbosity {
            0 => log::Level::Warn,
            1 => log::Level::Info,
            _ => log::Level::Debug,
        };

        unsafe {
            APP_LOGGER.0 = log_level;

            if let Err(err) = log::set_logger(&APP_LOGGER) {
                panic!("ERROR: {}", err)
            }
        }

        log::set_max_level(log_level.to_level_filter());

        Ok(App { path, net_id, page_id, place_id, transition_id, verbosity })
    }

    fn dump_page(&self, page: &Page) -> Result<(), DumpError> {
        let places = page.get_places();
        let mut place = None;

        if let Some(ref place_id) = self.place_id {
            if let Some(found_place) = places.iter().find(|p| p.get_id() == place_id) {
                info!("Found place \"{}\".", place_id);

                place = Some(found_place);
            } else {
                return Err(DumpError::PlaceNotFound(place_id.into(), page.get_id().into()))
            }
        } else if places.is_empty() {
            warn!("No places on page \"{}\".", page.get_id());
        }

        let transitions = page.get_transitions();
        let mut transition = None;

        if let Some(ref transition_id) = self.transition_id {
            if let Some(found_transition) = transitions.iter().find(|p| p.get_id() == transition_id)
            {
                info!("Found transition \"{}\".", transition_id);

                transition = Some(found_transition);
            } else {
                return Err(DumpError::TransitionNotFound(
                    transition_id.into(),
                    page.get_id().into(),
                ))
            }
        } else if transitions.is_empty() {
            warn!("No transitions on page \"{}\".", page.get_id());
        }

        if let Some(place) = place {
            println!("{:#?}", place);
        }

        if let Some(transition) = transition {
            println!("{:#?}", transition);
        }

        if place.is_none() && transition.is_none() {
            println!("{:#?}", page);
        }

        Ok(())
    }

    fn dump_net(&self, net: &Net) -> Result<(), DumpError> {
        let pages = net.get_pages();
        let page;

        if let Some(ref page_id) = self.page_id {
            if let Some(found_page) = net.get_pages().iter().find(|p| p.get_id() == page_id) {
                info!("Found page \"{}\".", page_id);
                page = Some(found_page);
            } else {
                return Err(DumpError::PageNotFound(page_id.into(), net.get_id().into()))
            }
        } else if pages.is_empty() {
            warn!("No pages in net \"{}\".", net.get_id());
            page = None;
        } else if pages.len() == 1 {
            page = pages.first();
        } else {
            page = None;
        }

        if let Some(page) = page {
            self.dump_page(page)
        } else {
            println!("{:#?}", net);
            Ok(())
        }
    }

    fn dump_nets(&self, nets: &[Net]) -> Result<(), DumpError> {
        let net;

        if let Some(ref net_id) = self.net_id {
            if let Some(found_net) = nets.iter().find(|n| n.get_id() == net_id) {
                info!("Found net \"{}\".", net_id);
                net = Some(found_net);
            } else {
                return Err(DumpError::NetNotFound(net_id.into(), self.path.display().to_string()))
            }
        } else if nets.is_empty() {
            warn!("No nets in PNML file \"{}\".", self.path.display());
            net = None;
        } else if nets.len() == 1 {
            net = nets.first();
        } else {
            net = None;
        }

        if let Some(net) = net {
            self.dump_net(net)
        } else {
            println!("{:#?}", nets);
            Ok(())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    let pnml = PNML::from_file(&app.path)?;

    for err in pnml.get_errors().iter() {
        warn!("[{}]: {}", err.text_start(), err);
    }

    if app.verbosity > 0 {
        if app.verbosity > 1 {
            debug!("{:?}", app);
        }
        info!("{:?}", pnml);
    }

    app.dump_nets(pnml.get_nets())?;

    Ok(())
}
