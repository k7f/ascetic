use ascetic_vis::TranslateScale;

#[derive(Clone, Copy, Debug)]
enum ZoomLevel {
    In(usize),
    Out(usize),
}

#[derive(Default, Debug)]
pub struct Zoom {
    level: Option<ZoomLevel>,
    ins:   Vec<f64>,
    outs:  Vec<f64>,
}

impl Zoom {
    pub fn new() -> Self {
        Zoom::default()
    }

    pub fn with_ins<I: IntoIterator<Item = f64>>(mut self, ins: I) -> Self {
        self.ins.clear();
        self.ins.extend(ins);

        self
    }

    pub fn with_outs<I: IntoIterator<Item = f64>>(mut self, outs: I) -> Self {
        self.outs.clear();
        self.outs.extend(outs);

        self
    }

    pub fn as_transform(&self) -> TranslateScale {
        let value = self.level.and_then(|level| match level {
            ZoomLevel::In(lvl) => self.ins.get(lvl).or_else(|| self.ins.last()).copied(),
            ZoomLevel::Out(lvl) => self.outs.get(lvl).or_else(|| self.outs.last()).copied(),
        });

        TranslateScale::scale(value.unwrap_or(1.))
    }

    #[inline]
    pub fn reset(&mut self) {
        self.level = None;
    }

    pub fn step_in(&mut self) -> Option<TranslateScale> {
        if let Some(level) = self.level {
            match level {
                ZoomLevel::In(lvl) => {
                    if let Some(v) = self.ins.get(lvl + 1) {
                        self.level = Some(ZoomLevel::In(lvl + 1));

                        Some(TranslateScale::scale(*v))
                    } else {
                        None
                    }
                }
                ZoomLevel::Out(lvl) => {
                    if lvl > 0 {
                        match self.outs.get(lvl - 1) {
                            Some(v) => {
                                self.level = Some(ZoomLevel::Out(lvl - 1));

                                Some(TranslateScale::scale(*v))
                            }
                            None => {
                                // FIXME log error
                                self.level = None;

                                Some(TranslateScale::scale(1.))
                            }
                        }
                    } else {
                        self.level = None;

                        Some(TranslateScale::scale(1.))
                    }
                }
            }
        } else if let Some(v) = self.ins.get(0) {
            self.level = Some(ZoomLevel::In(0));

            Some(TranslateScale::scale(*v))
        } else {
            None
        }
    }

    pub fn step_out(&mut self) -> Option<TranslateScale> {
        if let Some(level) = self.level {
            match level {
                ZoomLevel::In(lvl) => {
                    if lvl > 0 {
                        match self.ins.get(lvl - 1) {
                            Some(v) => {
                                self.level = Some(ZoomLevel::In(lvl - 1));

                                Some(TranslateScale::scale(*v))
                            }
                            None => {
                                // FIXME log error
                                self.level = None;

                                Some(TranslateScale::scale(1.))
                            }
                        }
                    } else {
                        self.level = None;

                        Some(TranslateScale::scale(1.))
                    }
                }
                ZoomLevel::Out(lvl) => {
                    if let Some(v) = self.outs.get(lvl + 1) {
                        self.level = Some(ZoomLevel::Out(lvl + 1));

                        Some(TranslateScale::scale(*v))
                    } else {
                        None
                    }
                }
            }
        } else if let Some(v) = self.outs.get(0) {
            self.level = Some(ZoomLevel::Out(0));

            Some(TranslateScale::scale(*v))
        } else {
            None
        }
    }
}
