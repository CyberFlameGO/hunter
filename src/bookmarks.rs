use termion::event::Key;

use std::collections::HashMap;

use crate::fail::{HResult, HError, ErrorLog};
use crate::widget::{Widget, WidgetCore};
use crate::files::{Files, File};
use crate::term;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Bookmarks {
    mapping: HashMap<char, String>,
}

impl Bookmarks {
    pub fn new() -> Bookmarks {
        let mut bm = Bookmarks { mapping: HashMap::new() };
        bm.load().log();
        bm
    }
    pub fn add(&mut self, key: char, path: &str) -> HResult<()> {
        self.mapping.insert(key, path.to_string());
        self.save()?;
        Ok(())
    }
    pub fn get(&self, key: char) -> HResult<&String> {
        let path = self.mapping.get(&key)?;
        Ok(path)
    }
    pub fn load(&mut self) -> HResult<()> {
        let bm_file = crate::paths::bookmark_path()?;
        let bm_content = std::fs::read_to_string(bm_file)?;


        let keys = bm_content.lines().step_by(2).map(|k| k);
        let paths = bm_content.lines().skip(1).step_by(2).map(|p| p);

        let mapping = keys.zip(paths).fold(HashMap::new(), |mut mapping, (key, path)| {
            if let Some(key) = key.chars().next() {
                let path = path.to_string();
                mapping.insert(key, path);
            }
            mapping
        });

        self.mapping = mapping;
        Ok(())
    }
    pub fn save(&self) -> HResult<()> {
        let bm_file = crate::paths::bookmark_path()?;
        let bookmarks = self.mapping.iter().map(|(key, path)| {
            format!("{}\n{}\n", key, path)
        }).collect::<String>();

        std::fs::write(bm_file, bookmarks)?;

        Ok(())
    }
}


pub struct BMPopup {
    core: WidgetCore,
    bookmarks: Bookmarks,
    bookmark_path: Option<String>,
    add_mode: bool,
}

impl BMPopup {
    pub fn new(core: &WidgetCore) -> BMPopup {
        let bmpopup = BMPopup {
            core: core.clone(),
            bookmarks: Bookmarks::new(),
            bookmark_path: None,
            add_mode: false
        };
        bmpopup
    }

    pub fn pick(&mut self, cwd: String) -> HResult<String> {
        self.bookmark_path = Some(cwd);
        self.refresh()?;
        self.popup()?;
        self.clear()?;

        let bookmark = self.bookmark_path.take();
        Ok(bookmark?)
    }

    pub fn add(&mut self, path: &str) -> HResult<()> {
        self.add_mode = true;
        self.bookmark_path = Some(path.to_string());
        self.refresh()?;
        self.clear()?;
        self.popup()?;
        self.clear()?;
        Ok(())
    }

    pub fn render_line(&self, n: u16, key: &char, path: &str) -> String {
        let xsize = term::xsize();
        let padding = xsize - 4;

        format!(
            "{}{}{}: {:padding$}",
            crate::term::goto_xy(1, n),
            crate::term::reset(),
            key,
            path,
            padding = padding as usize)
    }
}


impl Widget for BMPopup {
    fn get_core(&self) -> HResult<&WidgetCore> {
        Ok(&self.core)
    }
    fn get_core_mut(&mut self) -> HResult<&mut WidgetCore> {
        Ok(&mut self.core)
    }
    fn refresh(&mut self) -> HResult<()> {
        let tysize = crate::term::ysize();
        let txsize = crate::term::xsize();
        let len = self.bookmarks.mapping.len() as u16;
        let ysize = tysize - (len + 1);

        self.core.coordinates.set_position(1, ysize);
        self.core.coordinates.set_size(txsize, len+1);

        Ok(())
    }
    fn get_drawlist(&self) -> HResult<String> {
        let ypos = self.get_coordinates()?.ypos();

        let mut drawlist = String::new();

        if !self.add_mode {
            let cwd = self.bookmark_path.as_ref()?;
            drawlist += &self.render_line(ypos, &'`', cwd);
        }

        let bm_list = self.bookmarks.mapping.iter().enumerate().map(|(i, (key, path))| {
            let line = i as u16 + ypos + 1;
            self.render_line(line, key, path)
        }).collect::<String>();

        drawlist += &bm_list;

        Ok(drawlist)
    }
    fn on_key(&mut self, key: Key) -> HResult<()> {
        match key {
            Key::Ctrl('c') => {
                self.bookmark_path = None;
                return HError::popup_finnished()
            },
            Key::Char('`') => return HError::popup_finnished(),
            Key::Char(key) => {
                if self.add_mode {
                    let path = self.bookmark_path.take()?;
                    self.bookmarks.add(key, &path)?;
                    self.add_mode = false;
                    return HError::popup_finnished();
                }
                if let Ok(path) = self.bookmarks.get(key) {
                    self.bookmark_path.replace(path.clone());
                    return HError::popup_finnished();
                }
            }
            Key::Alt(key) => {
                self.bookmarks.mapping.remove(&key);
            }
            _ => {}
        }
        Ok(())
    }
}