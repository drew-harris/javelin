use clap::{Parser, Subcommand};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::{env, fs, process::Command};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Delete the entire database
    Clean,
    /// Open file 1 from the list
    #[command(name = "1")]
    One,
    /// Open file 2 from the list
    #[command(name = "2")]
    Two,
    /// Open file 3 from the list
    #[command(name = "3")]
    Three,
    /// Open file 4 from the list
    #[command(name = "4")]
    Four,
    /// Open file 5 from the list
    #[command(name = "5")]
    Five,
    /// Open file 6 from the list
    #[command(name = "6")]
    Six,
    /// Open file 7 from the list
    #[command(name = "7")]
    Seven,
    /// Open file 8 from the list
    #[command(name = "8")]
    Eight,
    /// Open file 9 from the list
    #[command(name = "9")]
    Nine,
}

fn open_file_by_index(index: usize) -> color_eyre::Result<()> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to determine data directory"))?
        .join("javelin");

    let db_path = data_dir.join("javelin.db");

    if !db_path.exists() {
        eprintln!("No files saved yet");
        return Ok(());
    }

    let db = PickleDb::load(
        &db_path,
        PickleDbDumpPolicy::DumpUponRequest,
        SerializationMethod::Json,
    )?;

    let current_dir = env::current_dir()?;
    let project_key = format!(
        "project_{}",
        current_dir.to_string_lossy().replace('/', "_")
    );

    if !db.exists(&project_key) {
        eprintln!("No files saved for this project");
        return Ok(());
    }

    let files: Vec<String> = db.get(&project_key).unwrap_or_default();

    if index >= files.len() {
        eprintln!(
            "File {} not found (only {} files saved)",
            index + 1,
            files.len()
        );
        return Ok(());
    }

    let file = &files[index];
    Command::new("zed").arg(file).status()?;

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Clean) => {
            let data_dir = dirs::data_dir()
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to determine data directory"))?
                .join("javelin");
            let db_path = data_dir.join("javelin.db");

            if db_path.exists() {
                fs::remove_file(&db_path)?;
                println!("Database deleted successfully");
            } else {
                println!("Database does not exist");
            }
            Ok(())
        }
        Some(Commands::One) => open_file_by_index(0),
        Some(Commands::Two) => open_file_by_index(1),
        Some(Commands::Three) => open_file_by_index(2),
        Some(Commands::Four) => open_file_by_index(3),
        Some(Commands::Five) => open_file_by_index(4),
        Some(Commands::Six) => open_file_by_index(5),
        Some(Commands::Seven) => open_file_by_index(6),
        Some(Commands::Eight) => open_file_by_index(7),
        Some(Commands::Nine) => open_file_by_index(8),
        None => {
            let terminal = ratatui::init();
            let result = App::new()?.run(terminal);
            ratatui::restore();
            result
        }
    }
}

pub struct App {
    /// Is the application running?
    running: bool,
    /// The database instance
    db: PickleDb,
    /// List state for navigation
    list_state: ListState,
    /// Cached file list
    files: Vec<String>,
    /// Current project key
    project_key: String,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to determine data directory"))?
            .join("javelin");

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        let db_path = data_dir.join("javelin.db");

        let db = if db_path.exists() {
            PickleDb::load(
                &db_path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Json,
            )?
        } else {
            PickleDb::new(
                &db_path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Json,
            )
        };

        let current_dir = env::current_dir()?;
        let project_key = format!(
            "project_{}",
            current_dir.to_string_lossy().replace('/', "_")
        );

        let mut app = Self {
            running: false,
            db,
            list_state: ListState::default(),
            files: Vec::new(),
            project_key,
        };

        app.load_files();
        Ok(app)
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        let current_dir = env::current_dir().unwrap_or_default();
        let items: Vec<ListItem> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                // Try to make the path relative to current directory for display
                let display_path = std::path::Path::new(file)
                    .strip_prefix(&current_dir)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| file.clone());

                let content = Line::from(vec![
                    Span::styled(
                        format!(
                            "{} ",
                            if i < 9 {
                                (i + 1).to_string()
                            } else {
                                " ".to_string()
                            }
                        ),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(display_path),
                ]);
                ListItem::new(content)
            })
            .collect();

        let project_name = env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "Unknown".to_string());

        let files_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Javelin - {} - Zed Files (Shift+J/K to reorder)",
                project_name
            )))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(files_list, chunks[0], &mut self.list_state);

        // Show current file that would be added with 'a'
        let current_file_info = if let Ok(file) = env::var("ZED_FILE") {
            // Display the relative path if possible
            let display_path = std::path::Path::new(&file)
                .strip_prefix(&current_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file.clone());

            if self.files.contains(&file) {
                format!("Current file: {} (already in list)", display_path)
            } else {
                format!("Press 'a' to add: {}", display_path)
            }
        } else {
            "No ZED_FILE environment variable set".to_string()
        };

        let info_paragraph = ratatui::widgets::Paragraph::new(current_file_info)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(info_paragraph, chunks[1]);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('j')) => self.next(),
            (_, KeyCode::Char('k')) => self.previous(),
            (_, KeyCode::Char('a')) => self.add_current_file(),
            (_, KeyCode::Char('d')) => self.delete_selected_file(),
            (KeyModifiers::SHIFT, KeyCode::Char('J')) => self.move_down(),
            (KeyModifiers::SHIFT, KeyCode::Char('K')) => self.move_up(),
            (_, KeyCode::Enter) => {
                if let Some(selected) = self.list_state.selected() {
                    self.open_file(selected);
                    self.quit();
                }
            }
            (_, KeyCode::Char(c)) if c.is_numeric() => {
                let index = c.to_digit(10).unwrap() as usize;
                if index > 0 && index <= self.files.len() {
                    self.open_file(index - 1);
                    self.quit();
                }
            }
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    /// Load files from the database
    fn load_files(&mut self) {
        if self.db.exists(&self.project_key) {
            self.files = self
                .db
                .get::<Vec<String>>(&self.project_key)
                .unwrap_or_default();
            if !self.files.is_empty() {
                self.list_state.select(Some(0));
            }
        }
    }

    /// Save files to the database
    fn save_files(&mut self) {
        self.db.set(&self.project_key, &self.files).unwrap();
    }

    /// Move to the next file in the list
    fn next(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Move to the previous file in the list
    fn previous(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Add the current file from ZED_FILE environment variable
    fn add_current_file(&mut self) {
        if let Ok(file) = env::var("ZED_FILE") {
            if !self.files.contains(&file) {
                self.files.push(file);
                self.save_files();
                if self.files.len() == 1 {
                    self.list_state.select(Some(0));
                }
            }
        }
    }

    /// Delete the currently selected file
    fn delete_selected_file(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.files.len() {
                self.files.remove(selected);
                self.save_files();

                if self.files.is_empty() {
                    self.list_state.select(None);
                } else if selected >= self.files.len() {
                    self.list_state.select(Some(self.files.len() - 1));
                }
            }
        }
    }

    /// Open a file at the given index with zed
    fn open_file(&self, index: usize) {
        if index < self.files.len() {
            let file = &self.files[index];

            // Use 'zed' command directly and ensure it completes
            let _ = Command::new("zed").arg(file).status();
        }
    }

    /// Move the selected file down in the list
    fn move_down(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.files.len() - 1 {
                self.files.swap(selected, selected + 1);
                self.save_files();
                self.list_state.select(Some(selected + 1));
            }
        }
    }

    /// Move the selected file up in the list
    fn move_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected > 0 {
                self.files.swap(selected, selected - 1);
                self.save_files();
                self.list_state.select(Some(selected - 1));
            }
        }
    }
}
