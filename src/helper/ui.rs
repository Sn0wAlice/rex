pub fn create_loading_bar() -> indicatif::ProgressBar {
    let spinner_style = indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .expect("static progress template")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let pb = indicatif::ProgressBar::new(1);
    pb.set_style(spinner_style);
    pb.set_prefix(format!("[{}/?]", 1));
    pb.set_message("Loading...");
    pb
}
