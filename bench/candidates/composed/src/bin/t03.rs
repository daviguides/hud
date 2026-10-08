use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

fn main() {
    let multi = MultiProgress::with_draw_target(ProgressDrawTarget::stdout());
    let style = ProgressStyle::with_template("{prefix:<15} {bar:30} {percent:>3}% {pos}/{len}")
        .unwrap()
        .progress_chars("━╸ ");
    let tasks = [("fetch index", 3), ("download crates", 120), ("verify", 12)];
    let bars: Vec<ProgressBar> = tasks
        .iter()
        .map(|(name, len)| {
            multi.add(
                ProgressBar::new(*len)
                    .with_style(style.clone())
                    .with_prefix(*name),
            )
        })
        .collect();
    for step in 0..120 {
        for (bar, (_, len)) in bars.iter().zip(&tasks) {
            if step < *len {
                bar.inc(1);
            }
        }
    }
    for bar in &bars {
        bar.finish();
    }
}
