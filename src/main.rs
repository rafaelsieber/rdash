use std::io;

mod config;
mod ui;

use ui::Dashboard;

fn main() -> io::Result<()> {
    let mut dashboard = Dashboard::new()?;
    dashboard.run()
}
