use crate::config::{Config, ProgramEntry};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::process::Command;

pub struct Dashboard {
    config: Config,
    selected_index: usize,
    mode: Mode,
    add_form: AddProgramForm,
    status_message: Option<String>,
    output_data: Option<(String, String)>, // (program_name, output)
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    AddProgram,
    Help,
    ShowOutput,
}

#[derive(Debug, Clone)]
struct AddProgramForm {
    step: usize,
    name: String,
    display_name: String,
    command: String,
    args: String,
    description: String,
    run_with_sudo: bool,
    show_output: bool,
}

impl AddProgramForm {
    fn new() -> Self {
        Self {
            step: 0,
            name: String::new(),
            display_name: String::new(),
            command: String::new(),
            args: String::new(),
            description: String::new(),
            run_with_sudo: false,
            show_output: false,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn current_field(&self) -> &str {
        match self.step {
            0 => "Program Name (identifier)",
            1 => "Display Name (what appears on dashboard)",
            2 => "Command (executable path or name)",
            3 => "Arguments (optional, space-separated)",
            4 => "Description (optional)",
            5 => "Run with sudo? (y/n)",
            6 => "Show output result? (y/n)",
            _ => "Review",
        }
    }

    fn current_value(&self) -> &str {
        match self.step {
            0 => &self.name,
            1 => &self.display_name,
            2 => &self.command,
            3 => &self.args,
            4 => &self.description,
            5 => if self.run_with_sudo { "y" } else { "n" },
            6 => if self.show_output { "y" } else { "n" },
            _ => "",
        }
    }

    fn set_current_value(&mut self, value: String) {
        match self.step {
            0 => self.name = value,
            1 => self.display_name = value,
            2 => self.command = value,
            3 => self.args = value,
            4 => self.description = value,
            5 => self.run_with_sudo = value.to_lowercase().starts_with('y'),
            6 => self.show_output = value.to_lowercase().starts_with('y'),
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.display_name.is_empty() && !self.command.is_empty()
    }
}

impl Dashboard {
    pub fn new() -> io::Result<Self> {
        let config = Config::load().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to load config: {}", e))
        })?;

        Ok(Self {
            config,
            selected_index: 0,
            mode: Mode::Normal,
            add_form: AddProgramForm::new(),
            status_message: None,
            output_data: None,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Enable raw mode and alternate screen
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;

        let result = self.main_loop();

        // Cleanup
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        terminal::disable_raw_mode()?;

        result
    }

    fn main_loop(&mut self) -> io::Result<()> {
        loop {
            self.draw()?;

            if let Event::Key(key) = event::read()? {
                match self.mode {
                    Mode::Normal => {
                        if self.handle_normal_mode(key)? {
                            break;
                        }
                    }
                    Mode::AddProgram => {
                        self.handle_add_program_mode(key)?;
                    }
                    Mode::Help => {
                        self.handle_help_mode(key);
                    }
                    Mode::ShowOutput => {
                        self.handle_show_output_mode(key);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('j') | KeyCode::Down => {
                let programs = self.config.get_programs();
                if !programs.is_empty() {
                    self.selected_index = (self.selected_index + 1) % programs.len();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let programs = self.config.get_programs();
                if !programs.is_empty() {
                    self.selected_index =
                        if self.selected_index == 0 { programs.len() - 1 } else { self.selected_index - 1 };
                }
            }
            KeyCode::Enter => {
                self.launch_selected_program()?;
            }
            KeyCode::Char('a') => {
                self.mode = Mode::AddProgram;
                self.add_form.reset();
            }
            KeyCode::Char('d') => {
                self.delete_selected_program()?;
            }
            KeyCode::Char('h') => {
                self.mode = Mode::Help;
            }
            KeyCode::Char('r') => {
                self.reload_config()?;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_add_program_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.add_form.reset();
            }
            KeyCode::Enter => {
                if self.add_form.step < 7 {
                    if self.add_form.step < 6 || self.add_form.is_complete() {
                        self.add_form.step += 1;
                        if self.add_form.step == 7 {
                            // Review step - save the program
                            self.save_new_program()?;
                            self.mode = Mode::Normal;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if self.add_form.step < 7 {
                    let mut current = self.add_form.current_value().to_string();
                    current.pop();
                    self.add_form.set_current_value(current);
                }
            }
            KeyCode::Char(c) => {
                if self.add_form.step < 7 {
                    if self.add_form.step == 5 || self.add_form.step == 6 {
                        // For sudo and show_output steps, only accept y/n
                        if c == 'y' || c == 'Y' || c == 'n' || c == 'N' {
                            self.add_form.set_current_value(c.to_string());
                        }
                    } else {
                        let mut current = self.add_form.current_value().to_string();
                        current.push(c);
                        self.add_form.set_current_value(current);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_show_output_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
                self.output_data = None;
            }
            _ => {}
        }
    }

    fn launch_selected_program(&mut self) -> io::Result<()> {
        let programs = self.config.get_programs();
        if let Some(program) = programs.get(self.selected_index) {
            if program.show_output {
                // Capture output
                let output = if program.run_with_sudo {
                    let mut cmd = Command::new("sudo");
                    cmd.arg(&program.command);
                    if !program.args.is_empty() {
                        cmd.args(&program.args);
                    }
                    cmd.output()
                } else {
                    let mut cmd = Command::new(&program.command);
                    if !program.args.is_empty() {
                        cmd.args(&program.args);
                    }
                    cmd.output()
                };

                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let combined_output = if stderr.is_empty() {
                            stdout.to_string()
                        } else if stdout.is_empty() {
                            stderr.to_string()
                        } else {
                            format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
                        };
                        
                        self.output_data = Some((program.display_name.clone(), combined_output));
                        self.mode = Mode::ShowOutput;
                        
                        if output.status.success() {
                            self.status_message = Some(format!("Executed: {}", program.display_name));
                        } else {
                            self.status_message = Some(format!("Executed with errors: {}", program.display_name));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error launching {}: {}", program.display_name, e));
                    }
                }
            } else {
                // Regular execution without capturing output
                // Save current terminal state
                execute!(io::stdout(), LeaveAlternateScreen, Show)?;
                terminal::disable_raw_mode()?;

                let result = if program.run_with_sudo {
                    // Handle sudo execution
                    let mut cmd = Command::new("sudo");
                    cmd.arg(&program.command);
                    if !program.args.is_empty() {
                        cmd.args(&program.args);
                    }
                    cmd.status()
                } else {
                    // Regular execution
                    let mut cmd = Command::new(&program.command);
                    if !program.args.is_empty() {
                        cmd.args(&program.args);
                    }
                    cmd.status()
                };

                // Restore terminal state
                terminal::enable_raw_mode()?;
                execute!(io::stdout(), EnterAlternateScreen, Hide)?;

                match result {
                    Ok(status) => {
                        if status.success() {
                            self.status_message = Some(format!("Executed: {}", program.display_name));
                        } else {
                            self.status_message = Some(format!("Failed to execute: {}", program.display_name));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error launching {}: {}", program.display_name, e));
                    }
                }
            }
        }
        Ok(())
    }

    fn delete_selected_program(&mut self) -> io::Result<()> {
        let programs = self.config.get_programs();
        if let Some(program) = programs.get(self.selected_index) {
            let name = program.name.clone();
            let display_name = program.display_name.clone();
            
            if self.config.remove_program(&name) {
                if let Err(e) = self.config.save() {
                    self.status_message = Some(format!("Error saving config: {}", e));
                } else {
                    self.status_message = Some(format!("Deleted: {}", display_name));
                    // Adjust selected index if necessary
                    let new_len = self.config.get_programs().len();
                    if new_len > 0 && self.selected_index >= new_len {
                        self.selected_index = new_len - 1;
                    }
                }
            }
        }
        Ok(())
    }

    fn save_new_program(&mut self) -> io::Result<()> {
        let args: Vec<String> = if self.add_form.args.is_empty() {
            vec![]
        } else {
            self.add_form.args.split_whitespace().map(|s| s.to_string()).collect()
        };

        let entry = ProgramEntry {
            name: self.add_form.name.clone(),
            display_name: self.add_form.display_name.clone(),
            command: self.add_form.command.clone(),
            args,
            description: if self.add_form.description.is_empty() {
                None
            } else {
                Some(self.add_form.description.clone())
            },
            run_with_sudo: self.add_form.run_with_sudo,
            show_output: self.add_form.show_output,
        };

        self.config.add_program(entry);
        
        if let Err(e) = self.config.save() {
            self.status_message = Some(format!("Error saving config: {}", e));
        } else {
            self.status_message = Some(format!("Added: {}", self.add_form.display_name));
        }

        self.add_form.reset();
        Ok(())
    }

    fn reload_config(&mut self) -> io::Result<()> {
        match Config::load() {
            Ok(config) => {
                self.config = config;
                self.selected_index = 0;
                self.status_message = Some("Configuration reloaded".to_string());
            }
            Err(e) => {
                self.status_message = Some(format!("Error reloading config: {}", e));
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        
        // Clear screen
        execute!(io::stdout(), Clear(ClearType::All))?;

        match self.mode {
            Mode::Normal => self.draw_main_screen(width, height)?,
            Mode::AddProgram => self.draw_add_program_screen(width, height)?,
            Mode::Help => self.draw_help_screen(width, height)?,
            Mode::ShowOutput => self.draw_output_screen(width, height)?,
        }

        io::stdout().flush()?;
        Ok(())
    }

    fn draw_main_screen(&self, width: u16, height: u16) -> io::Result<()> {
        // Draw top bar
        execute!(
            io::stdout(),
            MoveTo(0, 0),
            SetBackgroundColor(Color::Blue),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", " RDash - Server Dashboard", width = width as usize)),
            ResetColor
        )?;

        // Draw programs list
        let programs = self.config.get_programs();
        let start_y = 2;
        let content_height = height.saturating_sub(4); // Leave space for header and footer

        // Draw Sartre quote
        let quote = "\"L'homme est condamné à être libre.\" - Sartre";
        let quote_x = if width as usize > quote.len() {
            (width as usize - quote.len()) / 2
        } else {
            2
        } as u16;
        
        execute!(
            io::stdout(),
            MoveTo(quote_x, start_y),
            SetForegroundColor(Color::DarkGrey),
            Print(quote),
            ResetColor
        )?;

        let programs_start_y = start_y + 2;

        if programs.is_empty() {
            let empty_message = "No programs configured. Press 'a' to add a program.";
            let start_x = if width as usize > empty_message.len() { 
                (width as usize - empty_message.len()) / 2 
            } else { 
                2 
            } as u16;
            
            execute!(
                io::stdout(),
                MoveTo(start_x, programs_start_y + 2),
                Print(empty_message)
            )?;
        } else {
            // Calculate the maximum width needed for centering
            let max_program_width = programs.iter().map(|program| {
                let sudo_indicator = if program.run_with_sudo { " [SUDO]" } else { "" };
                let output_indicator = if program.show_output { " [OUT]" } else { "" };
                let display_text = if let Some(ref desc) = program.description {
                    format!("[ {}{}{} - {} ]", program.display_name, sudo_indicator, output_indicator, desc)
                } else {
                    format!("[ {}{}{} ]", program.display_name, sudo_indicator, output_indicator)
                };
                display_text.len()
            }).max().unwrap_or(0);

            let start_x = if width as usize > max_program_width { 
                (width as usize - max_program_width) / 2 
            } else { 
                2 
            } as u16;

            for (i, program) in programs.iter().enumerate() {
                if i < content_height as usize {
                    let y = programs_start_y + i as u16;
                    let is_selected = i == self.selected_index;

                    let sudo_indicator = if program.run_with_sudo { " [SUDO]" } else { "" };
                    let output_indicator = if program.show_output { " [OUT]" } else { "" };
                    let display_text = if let Some(ref desc) = program.description {
                        format!("[ {}{}{} - {} ]", program.display_name, sudo_indicator, output_indicator, desc)
                    } else {
                        format!("[ {}{}{} ]", program.display_name, sudo_indicator, output_indicator)
                    };

                    if is_selected {
                        execute!(
                            io::stdout(),
                            MoveTo(start_x, y),
                            SetBackgroundColor(Color::Yellow),
                            SetForegroundColor(Color::Black),
                            Print(&display_text),
                            ResetColor
                        )?;
                    } else {
                        execute!(
                            io::stdout(),
                            MoveTo(start_x, y),
                            Print(&display_text)
                        )?;
                    }
                }
            }
        }

        // Draw status message if any
        if let Some(ref message) = self.status_message {
            execute!(
                io::stdout(),
                MoveTo(2, height - 3),
                SetForegroundColor(Color::Green),
                Print(message),
                ResetColor
            )?;
        }

        // Draw bottom bar
        let help_text = "q:quit | j/k:↕ | Enter:launch | a:add | d:delete | h:help | r:reload";
        execute!(
            io::stdout(),
            MoveTo(0, height - 1),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", help_text, width = width as usize)),
            ResetColor
        )?;

        Ok(())
    }

    fn draw_add_program_screen(&self, width: u16, height: u16) -> io::Result<()> {
        // Draw top bar
        execute!(
            io::stdout(),
            MoveTo(0, 0),
            SetBackgroundColor(Color::Green),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", " Add New Program", width = width as usize)),
            ResetColor
        )?;

        let start_y = 3;

        // Draw form
        execute!(
            io::stdout(),
            MoveTo(2, start_y),
            Print(format!("Step {} of 7: {}", self.add_form.step + 1, self.add_form.current_field()))
        )?;

        execute!(
            io::stdout(),
            MoveTo(2, start_y + 2),
            Print(format!("> {}", self.add_form.current_value()))
        )?;

        if self.add_form.step == 7 {
            // Review step
            execute!(io::stdout(), MoveTo(2, start_y + 4), Print("Review:"))?;
            execute!(io::stdout(), MoveTo(4, start_y + 5), Print(format!("Name: {}", self.add_form.name)))?;
            execute!(io::stdout(), MoveTo(4, start_y + 6), Print(format!("Display: {}", self.add_form.display_name)))?;
            execute!(io::stdout(), MoveTo(4, start_y + 7), Print(format!("Command: {}", self.add_form.command)))?;
            if !self.add_form.args.is_empty() {
                execute!(io::stdout(), MoveTo(4, start_y + 8), Print(format!("Args: {}", self.add_form.args)))?;
            }
            if !self.add_form.description.is_empty() {
                execute!(io::stdout(), MoveTo(4, start_y + 9), Print(format!("Description: {}", self.add_form.description)))?;
            }
            execute!(io::stdout(), MoveTo(4, start_y + 10), Print(format!("Run with sudo: {}", if self.add_form.run_with_sudo { "Yes" } else { "No" })))?;
            execute!(io::stdout(), MoveTo(4, start_y + 11), Print(format!("Show output: {}", if self.add_form.show_output { "Yes" } else { "No" })))?;
            execute!(io::stdout(), MoveTo(2, start_y + 13), Print("Press Enter to save, Esc to cancel"))?;
        }

        // Draw bottom bar
        let help_text = "Enter:next | Esc:cancel | Type to input";
        execute!(
            io::stdout(),
            MoveTo(0, height - 1),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", help_text, width = width as usize)),
            ResetColor
        )?;

        Ok(())
    }

    fn draw_help_screen(&self, width: u16, height: u16) -> io::Result<()> {
        // Draw top bar
        execute!(
            io::stdout(),
            MoveTo(0, 0),
            SetBackgroundColor(Color::Magenta),
            SetForegroundColor(Color::White),
            Print(format!("{:width$}", " Help - RDash", width = width as usize)),
            ResetColor
        )?;

        let help_lines = vec![
            "",
            "RDash - Vim-like Server Dashboard",
            "",
            "NAVIGATION:",
            "  [ j ] [ ↓ ]        Move down",
            "  [ k ] [ ↑ ]        Move up",
            "  [ Enter ]          Launch selected program",
            "",
            "PROGRAM MANAGEMENT:",
            "  [ a ]              Add new program",
            "  [ d ]              Delete selected program",
            "  [ r ]              Reload configuration",
            "",
            "OTHER:",
            "  [ h ] [ F1 ]       Show this help",
            "  [ q ] [ Esc ]      Quit",
            "",
            "CONFIGURATION:",
            "  Config file: ~/.config/rdash/config.json",
            "  You can edit this file manually to modify programs",
            "",
            "Press any key to return...",
        ];

        // Calculate center position for content
        let content_width = help_lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        let start_x = if width > content_width { (width - content_width) / 2 } else { 2 };

        for (i, line) in help_lines.iter().enumerate() {
            if i + 2 < height as usize {
                execute!(
                    io::stdout(),
                    MoveTo(start_x, 2 + i as u16),
                    Print(line)
                )?;
            }
        }

        Ok(())
    }

    fn draw_output_screen(&self, width: u16, height: u16) -> io::Result<()> {
        if let Some((program_name, output)) = &self.output_data {
            // Draw top bar
            execute!(
                io::stdout(),
                MoveTo(0, 0),
                SetBackgroundColor(Color::Cyan),
                SetForegroundColor(Color::Black),
                Print(format!("{:width$}", format!(" Output: {}", program_name), width = width as usize)),
                ResetColor
            )?;

            // Draw output box border
            let box_width = width.saturating_sub(4);
            let box_height = height.saturating_sub(4);
            
            // Top border
            execute!(
                io::stdout(),
                MoveTo(1, 1),
                Print("┌"),
                Print("─".repeat(box_width as usize - 2)),
                Print("┐")
            )?;
            
            // Bottom border
            execute!(
                io::stdout(),
                MoveTo(1, height - 2),
                Print("└"),
                Print("─".repeat(box_width as usize - 2)),
                Print("┘")
            )?;
            
            // Side borders and content
            let output_lines: Vec<&str> = output.lines().collect();
            let content_height = box_height.saturating_sub(2) as usize;
            
            for i in 0..content_height {
                execute!(io::stdout(), MoveTo(1, 2 + i as u16), Print("│"))?;
                execute!(io::stdout(), MoveTo(box_width - 1, 2 + i as u16), Print("│"))?;
                
                if i < output_lines.len() {
                    let line = output_lines[i];
                    let max_content_width = (box_width.saturating_sub(4)) as usize;
                    let display_line = if line.len() > max_content_width {
                        format!("{}...", &line[0..max_content_width.saturating_sub(3)])
                    } else {
                        line.to_string()
                    };
                    execute!(io::stdout(), MoveTo(3, 2 + i as u16), Print(&display_line))?;
                }
            }

            // Draw bottom instruction
            execute!(
                io::stdout(),
                MoveTo(0, height - 1),
                SetBackgroundColor(Color::DarkGrey),
                SetForegroundColor(Color::White),
                Print(format!("{:width$}", " Press SPACE or ESC to close", width = width as usize)),
                ResetColor
            )?;
        }
        
        Ok(())
    }
}
