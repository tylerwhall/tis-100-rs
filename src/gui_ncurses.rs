extern crate ncurses;

use std::option::Option;
use std::iter::FromIterator;
use self::ncurses::*;
use cpu::ExecState;
use instruction;

struct CodeWin {
    wtext:  WINDOW,
    lines:  Vec<String>,
    line:   Option<u8>,
}

impl CodeWin {
    fn new(win: WINDOW, s: &str) -> Self {
        let wtext = derwin(win, TEXT_LINES, TEXT_COLS, 0, 0);
        wbkgd(wtext, '?' as u64);

        let mut codewin = CodeWin {
            wtext: wtext,
            lines: Self::code_vec(s),
            line:  None,
        };

        codewin.set_line(Some(1));
        codewin.draw_all_lines();
        codewin
    }

    fn code_vec(s: &str) -> Vec<String> {
        s.lines().map(str::to_string).collect()
    }

    fn set_code(&mut self, s: &str) {
        self.lines = Self::code_vec(s);
        self.draw_all_lines();
    }

    fn draw_all_lines(&mut self) {
        for i in (0..self.lines.len() as u8) {
            self.draw_line(i);
        }
    }

    fn draw_line(&mut self, line: u8) {
        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;
        getmaxyx(self.wtext, &mut max_y, &mut max_x);

        if self.line == Some(line) {
            wattr_on(self.wtext, A_STANDOUT());
        }

        let line = line as i32;
        assert!(line < max_y);

        wmove(self.wtext, line, 0);
        wclrtoeol(self.wtext);
        wmove(self.wtext, line, 0);
        if let Some(s) = self.lines.get(line as usize) {
            mvwprintw(self.wtext, line, 0, s);
            let pad: usize = TEXT_COLS as usize - s.len();
            wprintw(self.wtext, &String::from_iter([' '].iter().cloned().cycle().take(pad)));
        }
        wattr_off(self.wtext, A_STANDOUT());
        wrefresh(self.wtext);
    }

    /// Sets the active (highlighted) line
    fn set_line(&mut self, newline: Option<u8>) {
        let oldline = self.line;
        self.line = newline;

        if let Some(old) = oldline {
            self.draw_line(old);
        }

        if let Some(new) = newline {
            self.draw_line(new);
        }
    }
}

impl Drop for CodeWin {
    fn drop(&mut self) {
        delwin(self.wtext);
    }
}

struct CpuWin {
    win:        WINDOW,
    winner:     WINDOW,
    wsidebar:   WINDOW,
    codewin:    CodeWin,
}

const SIDEBAR_CELL_HEIGHT: i32 = 2;
const SIDEBAR_WIDTH: i32 = 6;
const CPUWIN_HEIGHT: i32 = (SIDEBAR_CELL_HEIGHT + 1) * 4 + 1;
const CPUWIN_WIDTH: i32 = CPUWIN_HEIGHT*2 + SIDEBAR_WIDTH;
const SIDEBAR_X: i32 = CPUWIN_WIDTH - 1 - SIDEBAR_WIDTH - 1;

const TEXT_LINES: i32 = CPUWIN_HEIGHT - 2;
const TEXT_COLS: i32 = CPUWIN_WIDTH - SIDEBAR_WIDTH - 3;

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
            codewin:    CodeWin::new(winner, "line1\nline2\n\nline4"),
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

    fn set_code(&mut self, s: &str) {
        self.codewin.set_code(s)
    }

    fn set_line(&mut self, newline: Option<u8>) {
        self.codewin.set_line(newline)
    }
}

impl Drop for CpuWin {
    fn drop(&mut self) {
        delwin(self.wsidebar);
        delwin(self.winner);
        wclear(self.win);
        //wrefresh(self.win);
        delwin(self.win);
    }
}

fn create_cpu_wins() -> Vec<Vec<CpuWin>> {
    let left_margin = 10;
    let inner_margin = 4;

    let mut cpuwins: Vec<Vec<CpuWin>> = (0..4).map(|x| {
        (0..3).map(|y| (x, y)).map(|(x, y)| {
            CpuWin::new(x*(CPUWIN_WIDTH + inner_margin) + left_margin,
                        y*(CPUWIN_HEIGHT + inner_margin/2) + left_margin/2)
        }).collect::<Vec<_>>()
    }).collect();

    for y in cpuwins.iter_mut() {
        for cpu in y.iter_mut() {
            cpu.refresh();
        }
    }

    cpuwins
}

pub fn gui() {
    initscr();
    refresh();

    let mut cpuwins = create_cpu_wins();

    loop {
        let c = getch();
        if c == b'q' as i32 {
            break;
        } else if c == KEY_RESIZE {
            drop(cpuwins);
            clear();
            refresh();
            cpuwins = create_cpu_wins();
        } else {
            cpuwins[0][0].set_code("whoo");
            refresh();
        }
    }

    drop(cpuwins);
    endwin();
}
