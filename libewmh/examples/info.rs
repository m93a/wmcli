use libewmh::prelude::*;

fn main() {
    let wm = WindowManager::connect().unwrap();
    let (_, wm_name) = wm.winmgr().unwrap();
    let win = wm.active_win().unwrap();
    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager:    {}", wm_name);
    println!("Composite Manager: {}", wm.composite_manager().unwrap());
    println!("Root Window:       {}", wm.root());
    println!("Work area:         {}x{}", wm.work_width(), wm.work_height());
    println!("Screen Size:       {}x{}", wm.width(), wm.height());
    println!("Desktops:          {}", wm.desktops().unwrap());
    println!();
    println!("Active Window");
    println!("{:-<120}", "");

    println!(
        "{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME"
    );

    let pid = wm.win_pid(win).unwrap_or(-1);
    let desktop = wm.win_desktop(win).unwrap_or(-1);
    let typ = wm.win_type(win).unwrap_or(WinType::Other(0));
    let states = wm.win_state(win).unwrap_or(vec![WinState::Other(0)]);
    let (x, y, w, h) = wm.win_geometry(win).unwrap_or((0, 0, 0, 0));
    let (l, r, t, b) = wm.win_borders(win).unwrap_or((0, 0, 0, 0));
    let class = wm.win_class(win).unwrap_or("".to_owned());
    let name = wm.win_name(win).unwrap_or("".to_owned());
    println!(
        "{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        format!("{:0>8}", win),
        format!("{:>2}", desktop),
        pid,
        format!("{:<4}", x),
        format!("{:<4}", y),
        format!("{:<4}", w),
        format!("{:<4}", h),
        format!("{},{},{},{}", l, r, t, b),
        typ.to_string(),
        format!("{:?}", states),
        class,
        name
    );
}
