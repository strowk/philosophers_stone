fn main() {
    supersolid::build::transpile_dir("assets/ui");
    superui_css_utilities::write_generated("assets/ui");
}