use nannou::prelude::*;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use warmy::{Load, Loaded, SimpleKey, Storage, Store, StoreOpt};

#[derive(Debug)]
enum Error {
    CannotLoadFromFS,
    CannotLoadFromLogical,
    IOError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::CannotLoadFromFS => f.write_str("cannot load from file system"),
            Error::CannotLoadFromLogical => f.write_str("cannot load from logical"),
            Error::IOError(ref e) => write!(f, "IO error: {}", e),
        }
    }
}

// The resource we want to take from a file.
#[derive(Debug)]
struct FromFS(String);

impl<C> Load<C, SimpleKey> for FromFS {
    type Error = Error;

    fn load(
        key: SimpleKey,
        _storage: &mut Storage<C, SimpleKey>,
        _: &mut C,
    ) -> Result<Loaded<Self, SimpleKey>, Self::Error> {
        // as we only accept filesystem here, we’ll ensure the key is a filesystem one
        match key {
            SimpleKey::Path(path) => {
                let mut fh = File::open(path).map_err(Error::IOError)?;
                let mut s = String::new();
                let _ = fh.read_to_string(&mut s);
                Ok(FromFS(s).into())
            }

            SimpleKey::Logical(_) => Err(Error::CannotLoadFromLogical),
        }
    }
}

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    Model::new()
}

struct Model {
    ctx: (),
    store: Store<(), SimpleKey>,
    file_content: String,
}

impl Model {
    pub fn new() -> Self {
        let mut ctx = ();
        let mut store: Store<(), SimpleKey> =
            Store::new(StoreOpt::default()).expect("store creation");
        let file_content = store
            // This only works if the test.rhai file is in the directory
            // where the script is ran
            .get::<FromFS>(&Path::new("test.rhai").into(), &mut ctx)
            .unwrap()
            .borrow()
            .0
            .clone();
        Model {
            ctx,
            store,
            file_content,
        }
    }

    pub fn update_content(&mut self) {
        self.store.sync(&mut self.ctx); // synchronize all resources (e.g. my_resource)
        self.file_content = self
            .store
            // This only works if the test.rhai file is in the directory
            // where the script is ran
            .get::<FromFS>(&Path::new("test.rhai").into(), &mut self.ctx)
            .unwrap()
            .borrow()
            .0
            .clone();
    }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.update_content();
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    let draw = app.draw();
    draw.background().color(WHITE);
    if m.file_content.contains("draw") {
        draw.background().color(RED);
    }
    draw.to_frame(app, &frame).unwrap();
    frame
}
