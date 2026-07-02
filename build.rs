// Встраивает иконку приложения в исполняемый файл Windows,
// чтобы у .exe и ярлыка на рабочем столе была наша иконка.
fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/hren_icon.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/hren_icon.ico");
        if let Err(e) = res.compile() {
            // Не валим сборку, но печатаем предупреждение в лог cargo.
            println!("cargo:warning=icon embed failed (need Windows SDK rc.exe): {e}");
        }
    }
}
