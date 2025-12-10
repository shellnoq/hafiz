//! Progress bar utilities for file transfers

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

/// Create a progress bar for file transfer
pub fn create_transfer_progress(total_bytes: u64, filename: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_bytes);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(filename.to_string());
    pb
}

/// Create a spinner for operations without known size
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Create a progress bar for counting items
pub fn create_counter(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Multi-progress for parallel transfers
pub struct TransferProgress {
    multi: MultiProgress,
    total_bar: ProgressBar,
}

impl TransferProgress {
    pub fn new(total_files: u64) -> Self {
        let multi = MultiProgress::new();
        let total_bar = multi.add(ProgressBar::new(total_files));
        total_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] Total: {pos}/{len} files ({percent}%)",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        Self { multi, total_bar }
    }

    /// Add a progress bar for a single file
    pub fn add_file(&self, total_bytes: u64, filename: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total_bytes));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.green} [{bar:30.cyan/blue}] {bytes}/{total_bytes} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(truncate_filename(filename, 30));
        pb
    }

    /// Increment total file count
    pub fn inc_total(&self) {
        self.total_bar.inc(1);
    }

    /// Finish all progress bars
    pub fn finish(&self) {
        self.total_bar.finish_with_message("Complete");
    }
}

/// Truncate filename for display
fn truncate_filename(filename: &str, max_len: usize) -> String {
    if filename.len() <= max_len {
        filename.to_string()
    } else {
        format!("...{}", &filename[filename.len() - max_len + 3..])
    }
}

/// Format bytes as human readable string
pub fn format_bytes(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

/// Format duration as human readable string
pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_filename() {
        assert_eq!(truncate_filename("short.txt", 20), "short.txt");
        assert_eq!(
            truncate_filename("this_is_a_very_long_filename.txt", 20),
            "...long_filename.txt"
        );
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1 MiB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }
}
