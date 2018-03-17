const KERNEL_PANIC_BANNER: &'static str = r#"
            (
       (      )     )
         )   (    (
        (          `
    .-""^"""^""^"""^""-.
  (//\\//\\//\\//\\//\\//)
   ~\^^^^^^^^^^^^^^^^^^/~
     `================`

  Oops, something went wrong

----------- PANIC ------------
"#;

#[no_mangle]
#[cfg(not(test))]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: ::std::fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    use console::kprintln;
    kprintln!("{}", KERNEL_PANIC_BANNER);
    kprintln!("FILE: {}", file);
    kprintln!("LINE: {}", line);
    kprintln!("COL: {}", col);
    kprintln!("");
    kprintln!("{}", fmt);

    loop { unsafe { asm!("wfe") } }
}

#[cfg(not(test))] #[lang = "eh_personality"] pub extern fn eh_personality() {}
