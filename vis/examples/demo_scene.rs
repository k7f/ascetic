use std::{
    fs::File,
    path::{Path, PathBuf},
    env::temp_dir,
    io::{Write, BufWriter},
    error::Error,
};
use piet::ImageFormat;
use ascetic_vis::{Scene, Theme, CairoBitmapDevice};

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
    const DEFAULT_PNG_COLOR_TYPE: png::ColorType = png::ColorType::RGBA;
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

    fn render_to_svg(&self, scene: &Scene, theme: &Theme) -> Result<Option<&Path>, Box<dyn Error>> {
        if let Some(ref svg_path) = self.svg_path {
            if self.verbosity > 0 {
                eprint!("Saving scene to \"{}\"...", svg_path.display());
            }

            let svg = scene.to_svg(theme, self.out_size, self.out_margin)?;
            let mut svg_file = File::create(svg_path)?;

            svg_file.write_all(&svg.into_bytes())?;

            Ok(Some(svg_path.as_path()))
        } else {
            Ok(None)
        }
    }

    fn render_to_png(&self, scene: &Scene, theme: &Theme) -> Result<&Path, Box<dyn Error>> {
        let out_width = self.out_size.0.round() as usize;
        let out_height = self.out_size.1.round() as usize;
        let mut device = CairoBitmapDevice::new(out_width, out_height, 1.)?;
        let mut rc = device.render_context();

        scene.render(theme, self.out_size, self.out_margin, &mut rc)?;

        if self.verbosity > 0 {
            eprintln!("Finished rendering.");
        }

        self.save_bitmap_image(device)
    }

    fn save_bitmap_image(&self, device: CairoBitmapDevice) -> Result<&Path, Box<dyn Error>> {
        if self.verbosity > 0 {
            eprint!("Writing image data to \"{}\"...", self.png_path.display());
        }

        if self.png_color_type != png::ColorType::RGBA || self.png_bit_depth != png::BitDepth::Eight
        {
            unimplemented!()
        }

        let pixels = device.into_raw_pixels(ImageFormat::RgbaPremul)?;
        let png_file = File::create(&self.png_path)?;
        let mut buf_writer = BufWriter::new(png_file);

        let out_width = self.out_size.0.round() as u32;
        let out_height = self.out_size.1.round() as u32;
        let mut encoder = png::Encoder::new(&mut buf_writer, out_width, out_height);
        encoder.set_color(self.png_color_type);
        encoder.set_depth(self.png_bit_depth);
        encoder.set_compression(self.png_compression.clone());
        encoder.set_filter(self.png_filter);

        encoder.write_header()?.write_image_data(pixels.as_slice())?;

        Ok(self.png_path.as_path())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    let mut theme = Theme::simple_demo();
    let scene = Scene::simple_demo(&theme);

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

    match app.render_to_svg(&scene, &theme) {
        Ok(None) => {}
        Ok(_) => {
            if app.verbosity > 0 {
                eprintln!(" Done.")
            }
        }
        Err(err) => {
            if app.verbosity > 0 {
                eprintln!(" Failed: {}.", err)
            }
        }
    }

    match app.render_to_png(&scene, &theme) {
        Ok(path) => {
            if app.verbosity > 0 {
                eprintln!(" Done.");
            }
            println!("{}", path.display());
            Ok(())
        }
        Err(err) => {
            if app.verbosity > 0 {
                eprintln!(" Failed: {}.", err);
            }
            Err(err)
        }
    }
}
