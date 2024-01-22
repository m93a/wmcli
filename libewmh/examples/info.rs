use libewmh::prelude::*;

fn main() {
    let wmcli = WindowManager::connect().unwrap();
    let (_, wm_name) = wmcli.winmgr().unwrap();
    let win = wmcli.active_win().unwrap();
    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager:    {}", wm_name);
    println!("Composite Manager: {}", wmcli.composite_manager().unwrap());
    println!("Root Window:       {}", wmcli.root());
    println!("Work area:         {}x{}", wmcli.work_width(), wmcli.work_height());
    println!("Screen Size:       {}x{}", wmcli.width(), wmcli.height());
    println!("Desktops:          {}", wmcli.desktops().unwrap());
    println!();
    println!("Active Window");
    println!("{:-<120}", "");

    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}", "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME");

    let pid = wmcli.win_pid(win).unwrap_or(-1);
    let desktop = wmcli.win_desktop(win).unwrap_or(-1);
    let typ = wmcli.win_type(win).unwrap_or(WinType::Invalid);
    let states = wmcli.win_state(win).unwrap_or(vec![WinState::Invalid]);
    let (x, y, w, h) = wmcli.win_geometry(win).unwrap_or((0,0,0,0));
    let (l, r, t, b) = wmcli.win_borders(win).unwrap_or((0, 0, 0, 0));
    let class = wmcli.win_class(win).unwrap_or("".to_owned());
    let name = wmcli.win_name(win).unwrap_or("".to_owned());
    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        format!("{:0>8}", win), format!("{:>2}", desktop), pid,
        format!("{:<4}", x), format!("{:<4}", y), format!("{:<4}", w), format!("{:<4}", h), 
        format!("{},{},{},{}", l, r, t, b),
        typ.to_string(), format!("{:?}", states), class, name);
}