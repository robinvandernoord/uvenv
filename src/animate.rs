use std::future::Future;
use std::io::{self, Write};
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
    pub fn default() -> AnimationSettings {
        AnimationSettings {
            style: AnimationStyle::Modern,
            order: AnimationOrder::Before,
        }
    }
}

fn get_spinner_chars(style: &AnimationStyle) -> Vec<char> {
    match style {
        AnimationStyle::Classic => {
            vec!['|', '/', '-', '\\']
        },
        AnimationStyle::Modern => vec!['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'],
    }
}

pub async fn animation(
    message: String,
    style: AnimationSettings,
) {
    let spinner_chars = get_spinner_chars(&style.style);
    let _order = &style.order;
    let mut idx = 0;
    loop {
        match &_order {
            AnimationOrder::Before => {
                eprint!("\r{} {} ", &spinner_chars[idx], &message);
            },
            AnimationOrder::After => {
                eprint!("\r{} {} ", &message, &spinner_chars[idx]);
            },
        };

        idx = (idx + 1) % spinner_chars.len();
        io::stdout().flush().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn show_loading_indicator<T, S: Into<String>>(
    promise: impl Future<Output = T>,
    message: S,
    style: AnimationSettings,
) -> T {
    let spinner = task::spawn(animation(message.into(), style));

    let result = promise.await;
    spinner.abort(); // Abort the spinner loop as download completes
    eprint!("\r\x1B[2K"); // clear the line

    result
}
