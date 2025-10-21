fn main() {
  unsafe {
    std::env::set_var("SLINT_NO_SYSTEM_THEME", "1");
  }
  slint_build::compile("ui/app.slint").unwrap();
}