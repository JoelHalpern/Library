use std::error::Error;

mod author_model;
mod book_model;
mod library_model;
mod library_window;

pub struct Config {
    argstr: String,
}

impl Config {
    // config extracting the file name parameter, if present,
    // from the cmd line parameters
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        let argstr = match args.next() {
            Some(_arg) => match args.next() {
                Some(parm) => parm,
                None => "".to_string(),
            },
            None => return Err("No program name??"),
        };

        Ok(Config { argstr })
    }
}

// Temporary "run".  Eventually will invoke the gui.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Config: {}", config.argstr);
    Ok(library_window::library_gui_base()?)
}
