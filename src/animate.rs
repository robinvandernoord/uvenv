use core::future::Future;
use core::iter::Cycle;
use core::slice::Iter;
use core::time::Duration;
use scopeguard::defer;
use std::io::{self, Write};
use tokio::task;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
pub enum AnimationStyle {
    #[default]
    Modern,
    #[expect(dead_code, reason = "Nice to have")]
    Classic,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
pub enum AnimationOrder {
    #[default]
    Before,
    #[expect(dead_code, reason = "Nice to have")]
    After,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
pub struct AnimationSettings {
    pub style: AnimationStyle,
    pub order: AnimationOrder,
}

impl AnimationSettings {
    pub fn get_spinner_chars(&self) -> Cycle<Iter<char>> {
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
