#[macro_use]
extern crate log;

use std::{
    collections::{HashSet, HashMap},
    path::PathBuf,
    error::Error,
};
use ascetic_pet::{PNML, Net, Page, PetItemKind, PetError};

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
    path:           PathBuf,
    item_selectors: HashMap<PetItemKind, String>,
    items_to_list:  HashSet<PetItemKind>,
    verbosity:      u32,
}

impl App {
    const DEFAULT_PATH: &'static str = "data/pnml/test.pnml";

    fn new() -> Result<Self, Box<dyn Error>> {
        let mut path = None;
        let mut item_selectors = HashMap::new();
        let mut items_to_list = HashSet::new();
        let mut verbosity = 0;

        for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
            match next_arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                "-vvv" => verbosity += 3,
                "--list-nets" => {
                    items_to_list.insert(PetItemKind::Net);
                }
                "--list-pages" => {
                    items_to_list.insert(PetItemKind::Page);
                }
                "--list-places" => {
                    items_to_list.insert(PetItemKind::Place);
                }
                "--list-transitions" => {
                    items_to_list.insert(PetItemKind::Transition);
                }
                "--list-arcs" => {
                    items_to_list.insert(PetItemKind::Arc);
                }
                "--net" | "--page" | "--place" | "--transition" => {}
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        match prev_arg.as_str() {
                            "--net" => {
                                item_selectors.insert(PetItemKind::Net, next_arg);
                            }
                            "--page" => {
                                item_selectors.insert(PetItemKind::Page, next_arg);
                            }
                            "--place" => {
                                item_selectors.insert(PetItemKind::Place, next_arg);
                            }
                            "--transition" => {
                                item_selectors.insert(PetItemKind::Transition, next_arg);
                            }
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

        Ok(App { path, item_selectors, items_to_list, verbosity })
    }

    fn dump_page(&self, page: &Page) -> Result<(), PetError> {
        let mut nothing_found = true;

        if let Some(place_id) = self.item_selectors.get(&PetItemKind::Place) {
            let place = page.get_place_by_id(place_id)?;

            info!("Found place \"{}\".", place_id);
            println!("{:#?}", place);

            nothing_found = false;
        } else if page.get_places().is_empty() {
            warn!("No places on page \"{}\".", page.get_id());
        }

        if let Some(transition_id) = self.item_selectors.get(&PetItemKind::Transition) {
            let transition = page.get_transition_by_id(transition_id)?;

            info!("Found transition \"{}\".", transition_id);
            println!("{:#?}", transition);

            nothing_found = false;
        } else if page.get_transitions().is_empty() {
            warn!("No transitions on page \"{}\".", page.get_id());
        }

        if nothing_found {
            println!("{:#?}", page);
        }

        Ok(())
    }

    fn dump_net(&self, net: &Net) -> Result<(), PetError> {
        let mut page = None;

        if let Some(page_id) = self.item_selectors.get(&PetItemKind::Page) {
            page = Some(net.get_page_by_id(page_id)?);

            info!("Found page \"{}\".", page_id);
        } else {
            let pages = net.get_pages();

            if pages.is_empty() {
                warn!("No pages in net \"{}\".", net.get_id());
            } else if pages.len() == 1 {
                page = pages.first();
            }
        }

        if let Some(page) = page {
            self.dump_page(page)
        } else {
            println!("{:#?}", net);
            Ok(())
        }
    }

    fn dump_items(&self, pnml: &PNML) -> Result<(), PetError> {
        let mut net = None;

        if let Some(net_id) = self.item_selectors.get(&PetItemKind::Net) {
            net = Some(pnml.get_net_by_id(net_id)?);

            info!("Found net \"{}\".", net_id);
        } else {
            let nets = pnml.get_nets();

            if nets.is_empty() {
                warn!("No nets in PNML file \"{}\".", self.path.display());
            } else if nets.len() == 1 {
                net = nets.first();
            }
        }

        if let Some(net) = net {
            self.dump_net(net)
        } else {
            println!("{:#?}", pnml.get_nets());
            Ok(())
        }
    }

    fn list_page_items(
        &self,
        page: &Page,
        stats: &mut HashMap<PetItemKind, usize>,
    ) -> Result<(), PetError> {
        if self.items_to_list.contains(&PetItemKind::Place) {
            let mut num_listed = stats.get(&PetItemKind::Place).copied().unwrap_or(0);

            for place in page.get_places() {
                num_listed += 1;
                println!("Place {}: {}", num_listed, place.get_id());
            }

            stats.insert(PetItemKind::Place, num_listed);
        }

        if self.items_to_list.contains(&PetItemKind::Transition) {
            let mut num_listed = stats.get(&PetItemKind::Transition).copied().unwrap_or(0);

            for transition in page.get_transitions() {
                num_listed += 1;
                println!("Transition {}: {}", num_listed, transition.get_id());
            }

            stats.insert(PetItemKind::Transition, num_listed);
        }

        Ok(())
    }

    fn list_net_items(
        &self,
        net: &Net,
        stats: &mut HashMap<PetItemKind, usize>,
    ) -> Result<(), PetError> {
        if self.items_to_list.contains(&PetItemKind::Page) {
            let mut num_listed = stats.get(&PetItemKind::Page).copied().unwrap_or(0);

            for page in net.get_pages() {
                num_listed += 1;
                println!("Page {}: {}", num_listed, page.get_id());
            }

            stats.insert(PetItemKind::Page, num_listed);
        }

        for page in net.get_pages() {
            self.list_page_items(page, stats)?;
        }

        Ok(())
    }

    fn list_items(&self, pnml: &PNML) -> Result<HashMap<PetItemKind, usize>, PetError> {
        let mut stats = HashMap::new();

        if self.items_to_list.contains(&PetItemKind::Net) {
            let mut num_listed = 0;

            for net in pnml.get_nets() {
                num_listed += 1;
                println!("Net {}: {}", num_listed, net.get_id());
            }

            stats.insert(PetItemKind::Net, num_listed);
        }

        for net in pnml.get_nets() {
            self.list_net_items(net, &mut stats)?;
        }

        Ok(stats)
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

    if app.items_to_list.is_empty() {
        app.dump_items(&pnml)?;
    } else {
        let stats = app.list_items(&pnml)?;
        info!("{:?}", stats);
    }

    Ok(())
}
