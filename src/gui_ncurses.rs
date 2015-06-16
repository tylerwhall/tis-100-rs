extern crate ncurses;

use std::option::Option;
use self::ncurses::*;
use cpu::ExecState;
use instruction;

struct CpuWin {
    win:        WINDOW,
    winner:     WINDOW,
    wsidebar:   WINDOW,
}

const SIDEBAR_CELL_HEIGHT: i32 = 2;
const SIDEBAR_WIDTH: i32 = 6;
const CPUWIN_HEIGHT: i32 = (SIDEBAR_CELL_HEIGHT + 1) * 4 + 1;
const CPUWIN_WIDTH: i32 = CPUWIN_HEIGHT*2 + SIDEBAR_WIDTH;
const SIDEBAR_X: i32 = CPUWIN_WIDTH - 1 - SIDEBAR_WIDTH - 1;

impl CpuWin {
    fn new(posx: i32, posy: i32) -> CpuWin {

        let win = newwin(CPUWIN_HEIGHT, CPUWIN_WIDTH, posy, posx);
        let winner = derwin(win, CPUWIN_HEIGHT-2, CPUWIN_WIDTH-2, 1, 1);
        let wsidebar = derwin(winner, CPUWIN_HEIGHT-2, SIDEBAR_WIDTH, 0, SIDEBAR_X);

        /* Border */
        box_(win, 0, 0);

        /* Left size of sidebar */
        mvwaddch(win, 0, SIDEBAR_X, ACS_TTEE());
        mvwvline(win, 1, SIDEBAR_X, ACS_VLINE(), CPUWIN_HEIGHT - 2);
        mvwaddch(win, CPUWIN_HEIGHT-1, SIDEBAR_X, ACS_BTEE());

        let mut cpuwin = CpuWin {
            win:        win,
            winner:     winner,
            wsidebar:   wsidebar,
        };
        cpuwin.cell_label(0, "ACC");
        cpuwin.cell_label(1, "BAK");
        cpuwin.cell_label(2, "LAST");
        cpuwin.cell_label(3, "MODE");

        cpuwin.cell_divider(1);
        cpuwin.cell_divider(2);
        cpuwin.cell_divider(3);

        cpuwin.set_values(0, 10, None, ExecState::EXEC);

        cpuwin
    }

    fn cell_label(&mut self, cell: i32, l: &str) {
        mvwprintw(self.wsidebar, Self::cell_y(cell), 1, l);
    }

    fn cell_val(&mut self, cell: i32, l: &str) {
        mvwprintw(self.wsidebar, Self::cell_y(cell)+1, 1, l);
    }

    fn cell_divider(&mut self, cell: i32) {
        mvwaddch(self.win, Self::cell_y(cell), SIDEBAR_X, ACS_LTEE());
        mvwhline(self.win, Self::cell_y(cell), SIDEBAR_X+1, ACS_HLINE(), SIDEBAR_WIDTH);
        mvwaddch(self.win, Self::cell_y(cell), SIDEBAR_X+SIDEBAR_WIDTH+1, ACS_RTEE());
    }

    fn cell_y(cell: i32) -> i32 {
        (SIDEBAR_CELL_HEIGHT + 1) * cell
    }

    fn refresh(&mut self) {
        wrefresh(self.win);
    }

    fn set_values(&mut self, acc: i32, bak: i32, last: Option<instruction::Port>, mode: ExecState) {
        let acc = format!("{:^4}", acc);
        let bak = format!("{:^4}", bak);
        let last = last.map_or_else(|| "N/A".to_string(), |p| p.to_string());
        let mode = mode.to_string();
        self.cell_val(0, &acc);
        self.cell_val(1, &bak);
        self.cell_val(2, &last);
        self.cell_val(3, &mode);
    }
}

impl Drop for CpuWin {
    fn drop(&mut self) {
        delwin(self.wsidebar);
        delwin(self.winner);
        wclear(self.win);
        wrefresh(self.win);
        delwin(self.win);
    }
}

pub fn gui() {
    initscr();
    refresh();

    let left_margin = 10;
    let inner_margin = 4;

    let mut cpuwins: Vec<Vec<CpuWin>> = (0..4).map(|x| {
        (0..3).map(|y| (x, y)).map(|(x, y)| {
            CpuWin::new(x*(CPUWIN_WIDTH + inner_margin) + left_margin,
                        y*(CPUWIN_HEIGHT + inner_margin/2) + left_margin/2)
        }).collect::<Vec<_>>()
    }).collect();

    refresh();
    getch();

    for y in cpuwins.iter_mut() {
        for cpu in y.iter_mut() {
            cpu.refresh();
        }
    }

    getch();

    refresh();
    getch();

    endwin();
}
