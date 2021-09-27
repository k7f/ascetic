use std::{
    fs::File,
    path::{Path, PathBuf},
    env::temp_dir,
    io::Write,
    error::Error,
};
use ascetic_vis::{Scene, Theme, backend::usvg::AsUsvgTree, backend::svg::ToSvg};

#[inline]
fn done_in_micros(start_time: Option<std::time::Instant>) {
    if let Some(elapsed) = start_time.map(|t| t.elapsed().as_micros()) {
        eprintln!(" Done ({} us).", elapsed);
    }
}

#[derive(Debug)]
struct App {
    png_path:         PathBuf,
    svg_path:         Option<PathBuf>,
    out_size:         (f64, f64),
    out_margin:       (f64, f64),
    png_color_type:   png::ColorType,
    png_bit_depth:    png::BitDepth,
    png_compression:  png::Compression,
    png_filter:       png::FilterType,
    theme_variation:  Option<String>,
    variation_amount: Option<f64>,
    #[allow(dead_code)]
    verbosity:        u32,
}

impl App {
    const DEFAULT_PNG_PATH: &'static str = "test.png";
    const DEFAULT_OUT_SIZE: (f64, f64) = (800., 450.);
    const DEFAULT_OUT_MARGIN: (f64, f64) = (10., 10.);
    const DEFAULT_PNG_COLOR_TYPE: png::ColorType = png::ColorType::Rgba;
    const DEFAULT_PNG_BIT_DEPTH: png::BitDepth = png::BitDepth::Eight;
    const DEFAULT_PNG_COMPRESSION: png::Compression = png::Compression::Fast;
    const DEFAULT_PNG_FILTER: png::FilterType = png::FilterType::NoFilter;

    fn new() -> Result<Self, Box<dyn Error>> {
        let mut png_path = None;
        let mut svg_path = None;
        let mut with_svg = false;
        let mut out_size = Self::DEFAULT_OUT_SIZE;
        let out_margin = Self::DEFAULT_OUT_MARGIN;
        let png_color_type = Self::DEFAULT_PNG_COLOR_TYPE;
        let png_bit_depth = Self::DEFAULT_PNG_BIT_DEPTH;
        let png_compression = Self::DEFAULT_PNG_COMPRESSION;
        let png_filter = Self::DEFAULT_PNG_FILTER;
        let mut theme_variation = None;
        let mut variation_amount = None;
        let mut verbosity = 0;

        for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
            match next_arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                "-vvv" => verbosity += 3,
                "--with-svg" => with_svg = true,
                "-w" | "-h" | "--svg" | "--theme" | "--amount" => {}
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        match prev_arg.as_str() {
                            "-w" => out_size.0 = arg.parse()?,
                            "-h" => out_size.1 = arg.parse()?,
                            "--svg" => svg_path = Some(PathBuf::from(arg)),
                            "--theme" => theme_variation = Some(next_arg),
                            "--amount" => variation_amount = Some(arg.parse()?),
                            _ => png_path = Some(PathBuf::from(arg)),
                        }
                    }
                }
            }
        }

        let png_path = png_path.unwrap_or_else(|| {
            let mut path = temp_dir();
            path.push(Self::DEFAULT_PNG_PATH);
            if verbosity > 0 {
                eprintln!("[WARN] Unspecified PNG output path; using \"{}\".", path.display());
            }
            path
        });

        if with_svg && svg_path.is_none() {
            svg_path = Some(png_path.clone().with_extension("svg"));
        }

        Ok(App {
            png_path,
            svg_path,
            out_size,
            out_margin,
            png_color_type,
            png_bit_depth,
            png_compression,
            png_filter,
            theme_variation,
            variation_amount,
            verbosity,
        })
    }

    #[inline]
    fn start(&self, message: &str) -> Option<std::time::Instant> {
        if self.verbosity > 0 {
            eprint!("{}", message);
            Some(std::time::Instant::now())
        } else {
            None
        }
    }

    fn render_to_svg(&self, scene: &mut Scene, theme: &Theme) -> Result<Option<&Path>, Box<dyn Error>> {
        if let Some(ref svg_path) = self.svg_path {
            let start_time = self.start("Rendering to svg...");
            let svg = scene.to_svg(theme, self.out_size, self.out_margin)?;
            done_in_micros(start_time);

            let start_time =
                self.start(format!("Saving scene to \"{}\"...", svg_path.display()).as_str());
            let mut svg_file = File::create(svg_path)?;
            svg_file.write_all(&svg.into_bytes())?;
            done_in_micros(start_time);

            Ok(Some(svg_path.as_path()))
        } else {
            Ok(None)
        }
    }

    fn render_to_png(&self, scene: &Scene, theme: &Theme) -> Result<&Path, Box<dyn Error>> {
        let start_time = self.start("Rendering to usvg...");
        let rtree = scene.as_usvg_tree(theme, self.out_size, self.out_margin)?;
        done_in_micros(start_time);

        self.save_bitmap_image(&rtree)
    }

    fn save_bitmap_image(&self, rtree: &usvg::Tree) -> Result<&Path, Box<dyn Error>> {
        let start_time = self.start("Rendering to bitmap...");
        let pixmap_size = rtree.svg_node().size.to_screen_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .expect("pixmap creation");
        resvg::render(rtree, usvg::FitTo::Original, pixmap.as_mut()).expect("pixmap rendering");
        done_in_micros(start_time);

        let start_time = self
            .start(format!("Writing image data to \"{}\"...", self.png_path.display()).as_str());
        pixmap.save_png(&self.png_path)?;
        done_in_micros(start_time);

        Ok(self.png_path.as_path())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    let mut theme = Theme::simple_demo();
    let mut scene = Scene::simple_demo(&theme);

    if app.verbosity > 1 {
        if app.verbosity > 2 {
            eprintln!("{:?}\n", app);
        }
        eprintln!("{:?}", scene);
    }

    if let Some(ref variation) = app.theme_variation {
        if let Some(amount) = app.variation_amount {
            theme.start_variation(Some(variation), 1);
            theme.step_variation(amount);
        } else {
            theme.use_variation(Some(variation));
        }
    }

    if app.verbosity > 2 {
        eprintln!("\n{:?}", theme);
    }

    app.render_to_svg(&mut scene, &theme)?;

    let path = app.render_to_png(&scene, &theme)?;
    println!("{}", path.display());

    Ok(())
}
