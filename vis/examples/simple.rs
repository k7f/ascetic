use std::{
    fs::File,
    path::{Path, PathBuf},
    env::temp_dir,
    io::{Write, BufWriter},
    error::Error,
};
use piet_common::{
    Device, BitmapTarget, ImageFormat, Color, UnitPoint,
    kurbo::{Line, Rect, RoundedRect, TranslateScale},
};
use ascetic_vis::{Scene, Group, Style, Stroke, Fill, Theme};

struct App {
    png_path:        PathBuf,
    svg_path:        Option<PathBuf>,
    out_size:        (f64, f64),
    out_margin:      (f64, f64),
    png_color_type:  png::ColorType,
    png_bit_depth:   png::BitDepth,
    png_compression: png::Compression,
    png_filter:      png::FilterType,
    #[allow(dead_code)]
    verbosity:       u32,
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
        let mut verbosity = 0;

        for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
            match next_arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                "--with-svg" => with_svg = true,
                "-w" | "-h" | "--svg" => {}
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        match prev_arg.as_str() {
                            "-w" => out_size.0 = next_arg.parse()?,
                            "-h" => out_size.1 = next_arg.parse()?,
                            "--svg" => svg_path = Some(PathBuf::from(arg)),
                            _ => png_path = Some(PathBuf::from(arg)),
                        }
                    }
                }
            }
        }

        let png_path = png_path.unwrap_or_else(|| {
            let mut path = temp_dir();
            path.push(Self::DEFAULT_PNG_PATH);
            eprintln!("[WARN] Unspecified PNG output path; using \"{}\".", path.display());
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
            verbosity,
        })
    }

    fn render_to_svg(&self, scene: &Scene, theme: &Theme) -> Result<Option<&Path>, Box<dyn Error>> {
        if let Some(ref svg_path) = self.svg_path {
            eprint!("Saving scene to \"{}\"...", svg_path.display());

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
        let mut device = Device::new()?;
        let mut bitmap = device.bitmap_target(out_width, out_height, 1.)?;
        let mut rc = bitmap.render_context();

        scene.render(theme, self.out_size, self.out_margin, &mut rc)?;

        eprintln!("Finished rendering.");

        self.save_bitmap_image(bitmap)
    }

    fn save_bitmap_image(&self, bitmap: BitmapTarget) -> Result<&Path, Box<dyn Error>> {
        eprint!("Writing image data to \"{}\"...", self.png_path.display());

        if self.png_color_type != png::ColorType::RGBA || self.png_bit_depth != png::BitDepth::Eight
        {
            unimplemented!()
        }

        let pixels = bitmap.into_raw_pixels(ImageFormat::RgbaPremul)?;
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

    let gradient_v_stops = vec![Color::WHITE, Color::BLACK];
    let gradient_h_stops = vec![Color::rgba8(0, 0xff, 0, 64), Color::rgba8(0xff, 0, 0xff, 64)];

    let gradients = vec![
        ("gradient-v", UnitPoint::TOP, UnitPoint::BOTTOM, gradient_v_stops.as_slice()),
        ("gradient-h", UnitPoint::LEFT, UnitPoint::RIGHT, gradient_h_stops.as_slice()),
    ];

    let styles = vec![
        ("border", Style::new().with_stroke(Stroke::new())),
        (
            "line-1",
            Style::new()
                .with_stroke(Stroke::new().with_brush(Color::rgb8(0, 0x80, 0x80)).with_width(3.)),
        ),
        (
            "line-2",
            Style::new()
                .with_stroke(Stroke::new().with_brush(Color::rgb8(0x80, 0x80, 0)).with_width(0.5)),
        ),
        ("rect-1", Style::new().with_fill(Fill::Linear("gradient-v".into()))),
        (
            "rect-2",
            Style::new()
                .with_fill(Fill::Linear("gradient-h".into()))
                .with_stroke(Stroke::new().with_brush(Color::BLACK).with_width(1.)),
        ),
    ];

    let theme = Theme::new().with_named_gradients(gradients).with_named_styles(styles);

    let mut scene = Scene::new((1000., 1000.));

    let border = scene.add_rect(Rect::new(0., 0., 1000., 1000.));
    let button = scene.add_rounded_rect(RoundedRect::new(250., 400., 450., 600., 10.));

    let lines = scene.add_grouped_lines(vec![
        (Line::new((0., 500.), (250., 0.)), theme.get("line-1")),
        (Line::new((0., 500.), (250., 1000.)), theme.get("line-1")),
        (Line::new((250., 1000.), (250., 0.)), theme.get("line-2")),
    ]);

    let rects = scene.add_group(Group::from_prims(vec![
        (button, theme.get("rect-1")),
        (button, theme.get("rect-2")),
    ]));

    let left_group = scene.add_group(Group::from_groups(vec![lines, rects]));

    scene.add_root(
        Group::from_prims(vec![(border, theme.get("border"))])
            .with_group(left_group)
            .with_group_ts(left_group, TranslateScale::translate((450., 0.).into())),
    );

    if app.verbosity > 1 {
        eprintln!("{:?}", scene);
    }

    match app.render_to_svg(&scene, &theme) {
        Ok(None) => {}
        Ok(_) => eprintln!(" Done."),
        Err(err) => eprintln!(" Failed: {}.", err),
    }

    match app.render_to_png(&scene, &theme) {
        Ok(path) => {
            eprintln!(" Done.");

            println!("{}", path.display());

            Ok(())
        }
        Err(err) => {
            eprintln!(" Failed: {}.", err);

            Err(err)
        }
    }
}
