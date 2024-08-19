use scopeguard::defer;
use std::future::Future;
use std::io::{self, Write};
use std::iter::Cycle;
use std::time::Duration;
use tokio::task;

#[allow(dead_code)]
pub enum AnimationStyle {
    Classic,
    Modern,
}

#[allow(dead_code)]
pub enum AnimationOrder {
    Before,
    After,
}

pub struct AnimationSettings {
    pub style: AnimationStyle,
    pub order: AnimationOrder,
}

impl AnimationSettings {
    pub const fn default() -> Self {
        Self {
            style: AnimationStyle::Modern,
            order: AnimationOrder::Before,
        }
    }

    pub fn get_spinner_chars(&self) -> Cycle<std::slice::Iter<char>> {
        match self.style {
            AnimationStyle::Classic => ['|', '/', '-', '\\'].iter(),
            AnimationStyle::Modern => ['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'].iter(),
        }
        .cycle()
    }
}

pub async fn animation(
    message: String,
    style: AnimationSettings,
) -> Option<()> {
    let mut spinner_chars = style.get_spinner_chars();
    let ordering = &style.order;
    loop {
        let char = spinner_chars.next()?; // Cycle never returns None, but ? required for type
        match &ordering {
            AnimationOrder::Before => {
                eprint!("\r{} {} ", &char, &message);
            },
            AnimationOrder::After => {
                eprint!("\r{} {} ", &message, &char);
            },
        };

        io::stdout().flush().expect("Writing to stdout failed?");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn show_loading_indicator<T, S: Into<String>, P: Future<Output = T>>(
    promise: P,
    message: S,
    style: AnimationSettings,
) -> T {
    let spinner = task::spawn(animation(message.into(), style));

    defer! {
        spinner.abort(); // Abort the spinner loop as download completes
        eprint!("\r\x1B[2K"); // clear the line
    }

    promise.await
}
