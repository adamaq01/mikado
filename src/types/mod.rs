use crate::helpers;
use std::fmt::{Display, Formatter};

pub mod cloudlink;
pub mod game;
pub mod tachi;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameProperties {
    model: Box<str>,
    dest: Box<str>,
    spec: Box<str>,
    revision: Box<str>,
    ext: u64,
    // Derived props
    valkyrie: bool,
    maxxive_support: bool,
    ultimate_support: bool,
}

pub enum NotSupportedReason<'a> {
    WrongModel(&'a str),
    OmnimixDetected,
    TooOld(u64),
}

#[allow(unused)]
impl GameProperties {
    pub unsafe fn from_ea3_node(node: *const ()) -> Option<GameProperties> {
        let model =
            unsafe { helpers::read_node_str(node, c"/soft/model".as_ptr(), 3) }?.into_boxed_str();
        let dest =
            unsafe { helpers::read_node_str(node, c"/soft/dest".as_ptr(), 1) }?.into_boxed_str();
        let spec =
            unsafe { helpers::read_node_str(node, c"/soft/spec".as_ptr(), 1) }?.into_boxed_str();
        let revision =
            unsafe { helpers::read_node_str(node, c"/soft/rev".as_ptr(), 1) }?.into_boxed_str();
        let ext = unsafe { helpers::read_node_str(node, c"/soft/ext".as_ptr(), 10) }?
            .parse()
            .unwrap_or(0);

        let valkyrie = spec.as_ref() == "G" || spec.as_ref() == "H";
        let maxxive_support = ext >= 2025042200;
        let ultimate_support = ext >= 2025062400; // Actually it is 2025062401 but let's be more lenient

        Some(GameProperties {
            model,
            dest,
            spec,
            revision,
            ext,
            valkyrie,
            maxxive_support,
            ultimate_support,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn dest(&self) -> &str {
        &self.dest
    }

    pub fn spec(&self) -> &str {
        &self.spec
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn ext(&self) -> u64 {
        self.ext
    }

    pub fn is_valkyrie(&self) -> bool {
        self.valkyrie
    }

    pub fn has_maxxive_support(&self) -> bool {
        self.maxxive_support
    }
    
    pub fn has_ultimate_support(&self) -> bool {
        self.ultimate_support
    }

    pub fn is_not_supported(&self) -> Option<NotSupportedReason<'_>> {
        if self.model() != "KFC" {
            Some(NotSupportedReason::WrongModel(self.model()))
        } else if self.dest() == "O" || self.dest() == "X" {
            Some(NotSupportedReason::OmnimixDetected)
        } else if self.ext() < 2022083000 {
            Some(NotSupportedReason::TooOld(self.ext()))
        } else {
            None
        }
    }
}

impl Display for GameProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}:{}:{}:{}",
            self.model, self.dest, self.spec, self.revision, self.ext
        ))?;
        if self.valkyrie {
            f.write_str(" (Valkyrie)")?;
        }
        if self.maxxive_support {
            f.write_str(" (Maxxive support)")?;
        }
        if self.ultimate_support {
            f.write_str(" (Ultimate support)")?;
        }
        Ok(())
    }
}

impl Display for NotSupportedReason<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NotSupportedReason::WrongModel(model) => {
                write!(f, "Game model '{model}' is not related to chicken")
            }
            NotSupportedReason::OmnimixDetected => write!(f, "Omnimix/Plus detected"),
            NotSupportedReason::TooOld(ext) => write!(f, "Game version '{ext}' is too old"),
        }
    }
}
