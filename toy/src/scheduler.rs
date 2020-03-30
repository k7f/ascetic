use std::{
    collections::HashSet,
    hash::Hash,
    iter::FromIterator,
    time::{Instant, Duration},
    thread,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    UpdateWindow,
    UpdateKeys,
    UpdateMouse,
    RenderScene,
    RedrawContents,
    ModifyTheme,
    Pan,
    Zoom,
    Exit,
    DoNothing,
}

#[derive(Clone, Copy, Default, Debug)]
struct Debouncer {
    anchor: Option<Instant>,
    period: Duration,
}

impl Debouncer {
    fn new(period: Duration) -> Self {
        Debouncer { anchor: None, period }
    }

    #[inline]
    fn set_period(&mut self, period: Duration) {
        self.period = period;
    }

    #[inline]
    fn check(&mut self) -> Option<Duration> {
        self.anchor.and_then(|then| {
            self.period.checked_sub(then.elapsed()).or_else(|| {
                self.anchor = None;
                None
            })
        })
    }

    #[inline]
    fn start(&mut self) {
        if self.anchor.is_none() {
            self.anchor = Some(Instant::now());
        }
    }

    #[inline]
    fn is_started(&self) -> bool {
        self.anchor.is_some()
    }
}

#[derive(Debug)]
struct Queue<T: Eq + Hash> {
    items: HashSet<T>, // FIXME Vec<Option<Priority>>
}

impl<T: Eq + Hash> Queue<T> {
    #[inline]
    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }

    // FIXME calculate Priority based on debouncers state
    #[inline]
    pub fn insert(&mut self, item: T) -> bool {
        self.items.insert(item)
    }

    #[inline]
    pub fn remove(&mut self, item: &T) -> bool {
        self.items.remove(item)
    }
}

impl<T: Eq + Hash> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let items = HashSet::from_iter(items);

        Queue { items }
    }
}

#[derive(Debug)]
pub struct Scheduler {
    actions:    Queue<Action>,
    debouncers: Vec<Debouncer>,
}

impl Scheduler {
    pub fn with_debouncers<I: IntoIterator<Item = (Action, Duration)>>(mut self, items: I) -> Self {
        for (action, period) in items.into_iter() {
            let pos = action as usize;

            self.debouncers.resize_with(pos + 1, Default::default);
            self.debouncers[pos].set_period(period);
        }

        self
    }

    #[inline]
    pub fn enroll(&mut self, action: Action) -> bool {
        if let Some(debouncer) = self.debouncers.get_mut(action as usize) {
            debouncer.start();
        }

        self.actions.insert(action)
    }

    #[inline]
    pub fn is_pending(&self, action: Action) -> bool {
        self.actions.contains(&action)
    }

    #[inline]
    pub fn is_ready(&mut self, action: Action) -> bool {
        self.sleep_period(action).is_none() && self.actions.remove(&action)
    }

    #[inline]
    pub fn sleep_period(&mut self, action: Action) -> Option<Duration> {
        self.debouncers.get_mut(action as usize).and_then(|d| d.check())
    }

    pub fn next_eager(&mut self) -> Option<Action> {
        if self.is_ready(Action::UpdateWindow) {
            Some(Action::UpdateWindow)
        } else if self.is_ready(Action::RenderScene) {
            Some(Action::RenderScene)
        } else if self.is_ready(Action::RedrawContents) {
            Some(Action::RedrawContents)
        } else if self.is_ready(Action::ModifyTheme) {
            Some(Action::ModifyTheme)
        } else if self.is_ready(Action::Pan) {
            Some(Action::Pan)
        } else if self.is_ready(Action::Zoom) {
            Some(Action::Zoom)
        } else if self.is_ready(Action::UpdateKeys) {
            Some(Action::UpdateKeys)
        } else if self.is_ready(Action::UpdateMouse) {
            Some(Action::UpdateMouse)
        } else {
            None
        }
    }

    pub fn next_lazy(&mut self) -> Option<Action> {
        // FIXME pick the shortest sleeper
        let action = Action::UpdateWindow;

        if let Some(sleep_period) = self.sleep_period(action) {
            thread::sleep(sleep_period);

            if self.actions.remove(&action) {
                Some(action)
            } else {
                // FIXME log
                None
            }
        } else {
            // no sleepers
            None
        }
    }

    pub fn next_action(&mut self) -> Option<Action> {
        self.next_eager().or_else(|| self.next_lazy())
    }
}

impl FromIterator<Action> for Scheduler {
    fn from_iter<I: IntoIterator<Item = Action>>(actions: I) -> Self {
        let actions = Queue::from_iter(actions);
        let debouncers = vec![Debouncer::default(); Action::DoNothing as usize];

        Scheduler { actions, debouncers }
    }
}
