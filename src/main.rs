use std::process;
use std::io::prelude::*;
use std::io;

// Rust program for GPM
struct GPM {
    st: Box<[usize]>,
    s: usize,
    e: usize,
    q: usize,
    c: usize,
    h: usize,
    p: usize,
    f: usize,
    a: usize,
    w: usize,
    marker: usize,
    machine_macro: Vec<fn(&mut GPM)>,
}

impl GPM {
    // n is stack size allowed. This should be as large as
    // possible -- say 10,000.
    fn new(n: usize) -> Self {
        let machine_macro = vec![
            Self::def as fn(&mut GPM),
            Self::val,
            Self::update,
            Self::bin,
            Self::dec,
            Self::bar,
            ];

        let mut st = vec![0; n].into_boxed_slice();
        let mst = vec![
            -1i8 as u8, 4, b'D', b'E', b'F', -1i8 as u8,
            0, 4, b'V', b'A', b'L', -2i8 as u8,
            6, 7, b'U', b'P', b'D', b'A', b'T', b'E', -3i8 as u8,
            12, 4, b'B', b'I', b'N', -4i8 as u8,
            21, 4, b'D', b'E', b'C', -5i8 as u8,
            27, 4, b'B', b'A', b'R', -6i8 as u8,
        ];
        // The name-value pairs for the six machine code macros
        // are first assembled in the vector mst and then copied
        // to the base of the stack.
        for (i, val) in mst.into_iter().enumerate() {
            st[i] = val as usize;
        }

        Self {
            st,
            s: 39,
            e: 33,
            q: 1,
            c: 0,
            h: 0,
            p: 0,
            f: 0,
            a: 0,
            w: 0,
            marker: (-2isize).pow(20) as usize,
            machine_macro,
        }
    }

    fn load(&mut self) {
        if self.h == 0 {
            write_symbol(&self.a)
        } else {
            self.st[self.s] = self.a;
            self.s += 1;
        }
    }

    fn next_ch(&mut self) {
        if self.c == 0 {
            read_symbol(&mut self.a);
        } else {
            self.a = self.st[self.c];
            self.c += 1;
        }
    }

    fn find(&mut self, x: usize) {
        self.w = x;
        self.a = self.e;
        let mut flag = false;
        'next: while (self.a as isize) > 0 {
            if flag {
                self.a = self.st[self.a];
            }
            flag = true;
            for r in 0..self.st[self.w] {
                if self.st[self.w + r] != self.st[self.a + r + 1] {
                    continue 'next;
                }
            }
            self.w = self.a + 1 + self.st[self.w];
            return;
        }
        self.monitor7()
    }

    // This routine depends on the method for marking machine
    // code macros. The method adopted here (which is
    // different from that described in the paper or used in the
    // actual Titan program) is to make the value a negative
    // index integer which is used to index the label vector
    // machine_macro, whose entries are the labels of the
    // corresponding programs.
    fn jump_if_marked(&mut self, x: usize) {
        if -6 < (x as i8) && (x as i8) < 0 {
            self.machine_macro[-(x as i8) as usize - 1](self);
        }
    }

    // Main cycle
    fn start(&mut self) {
        self.next_ch();
        //eprintln!("{:?}", self.a as u8 as char);
        match self.a as u8 {
            b'<' => {
                self.q += 1;
                self.q2()
            }
            b'$' => self.fn_(),
            b',' => self.next_item(),
            b';' => self.apply(),
            b'~' => self.load_arg(),
            _ if self.a == self.marker => self.end_fn(),
            b'>' => self.exit(),
            _ => self.copy(),
        }
    }

    fn copy(&mut self) {
        self.load();
        if self.q == 1 {
            self.start()
        } else {
            self.q2()
        }
    }

    fn q2(&mut self) {
        self.next_ch();
        if self.a as u8 == b'<' {
            self.q += 1;
            return self.copy();
        } else if self.a as u8 != b'>' {
            return self.copy();
        }
        self.q -= 1;
        if self.q == 1 {
            self.start()
        } else {
            self.copy()
        }
    }

    // Warning character actions

    fn fn_(&mut self) {
        self.st[self.s] = self.h;
        self.st[self.s + 1] = self.f;
        self.st[self.s + 2] = 0;
        self.st[self.s + 3] = 0;
        self.h = self.s + 3;
        self.f = self.s + 1;
        self.s += 4;
        self.start()
    }

    fn next_item(&mut self) {
        if self.h == 0 {
            return self.copy();
        }
        self.st[self.s] = 0;
        self.st[self.h] = self.s - self.h - self.st[self.h];
        self.h = self.s;
        self.s += 1;
        self.start()
    }

    fn apply(&mut self) {
        if self.p > self.f {
            return self.monitor1();
        } else if self.h == 0 {
            return self.copy();
        }
        let stf = self.st[self.f];
        let stfm1 = self.st[self.f - 1];
        self.st[self.f + 1] = self.c;
        self.st[self.f] = self.p;
        self.st[self.f - 1] = self.s - self.f + 2;
        self.st[self.s] = self.marker;
        self.st[self.h] = self.s - self.h;
        self.s += 1;
        self.h = stfm1;
        self.p = self.f;
        self.f = stf;
        if self.h != 0 {
            self.st[self.h] += self.st[self.p - 1];
        }
        self.find(self.p + 2);
        self.jump_if_marked(self.st[self.w]);
        self.c = self.w + 1;
        self.start()
    }

    fn load_arg(&mut self) {
        if self.p == 0 {
            if self.h == 0 {
                return self.copy();
            } else {
                return self.monitor2();
            }
        }
        self.next_ch();
        self.w = self.p + 2;
        if (number(self.a) as isize) < 0 {
            return self.monitor3();
        }
        for _ in 0..number(self.a) {
            self.w += self.st[self.w];
            if self.st[self.w] == self.marker {
                return self.monitor4();
            }
        }
        for r in 1..self.st[self.w] {
            self.a = self.st[self.w + r];
            self.load();
        }
        self.start()
    }

    fn end_fn(&mut self) {
        if self.f > self.p {
            return self.monitor5();
        }

        self.a = self.s;
        self.st[self.s] = self.e;

        while self.st[self.a] >= self.p - 1 + self.st[self.p - 1] {
            let sta = self.st[self.a];
            self.st[self.a] -= self.st[self.p - 1];
            self.a = sta;
        }

        self.w = self.st[self.a];

        while self.w >= self.p - 1 {
            self.w = self.st[self.w];
        }

        self.st[self.a] = self.w;
        self.e = self.st[self.s];

        if self.h != 0 {
            if self.h > self.p {
                self.h -= self.st[self.p - 1];
            } else {
                self.st[self.h] -= self.st[self.p - 1];
            }
        }

        self.w = self.p - 1 + self.st[self.p - 1];
        self.a = self.p - 1;
        self.s -= self.st[self.p - 1];
        self.c = self.st[self.p + 1];
        self.p = self.st[self.p];

        while self.a != self.s {
            self.st[self.a] = self.st[self.w];
            self.w += 1;
            self.a += 1;
        }
        self.start()
    }

    fn exit(&mut self) {
        if self.c != 0 || self.h != 0 {
            return self.monitor8();
        }
        process::exit(0);
    }

    // Machine code macros

    // This version of DEF is shorter than that given in 
    // Section 7 as it leaves `end_fn` to copy back the definition
    fn def(&mut self) {
        if self.h != 0 {
            self.st[self.h] -= self.st[self.p - 1] - 6;
        }
        self.st[self.p - 1] = 6;
        self.st[self.p + 5] = self.e;
        self.e = self.p + 5;
        self.end_fn()
    }

    fn val(&mut self) {
        self.find(self.p + 6);
        while self.st[self.w + 1] != self.marker {
            self.w += 1;
            self.a = self.st[self.w];
            self.load();
        }
        self.end_fn();
    }

    fn update(&mut self) {
        self.find(self.p + 9);
        self.a = self.p + 9 + self.st[self.p + 9];
        if self.st[self.a] > self.st[self.w] {
            return self.monitor9();
        }
        for r in 1..=self.st[self.a] {
            self.st[self.w + r] = self.st[self.a + r];
        }
        self.end_fn();
    }

    fn bin(&mut self) {
        self.w = 0;
        self.a = if self.st[self.p + 7] == b'+' as usize
            || self.st[self.p + 7] == b'-' as usize {
            self.p + 8
        } else {
            self.p + 7
        };
        while self.st[self.a] != self.marker {
            let x = number(self.st[self.a]);
            if !(0..=9).contains(&x) {
                self.monitor10();
            }
            self.w = self.w * 10 + x;
            self.a += 1;
        }
        self.st[self.s] = if self.st[self.p + 7] == b'-' as usize {
            -(self.w as isize) as usize
        } else {
            self.w
        };
        self.s += 1;
    }

    fn dec(&mut self) {
        self.w = self.st[self.p + 7];
        if (self.w as isize) < 0 {
            self.w = -(self.w as isize) as usize;
            self.a = b'-' as usize;
            self.load();
        }
        let mut w1 = 1;
        while 10 * w1 <= self.w {
            w1 *= 10;
        }
        loop {
            self.a = char_(self.w / w1);
            self.w %= w1;
            w1 /= 10;
            if w1 < 1 {
                break;
            }
        }
        self.end_fn();
    }

    fn bar(&mut self) {
        self.w = self.st[self.p + 9];
        self.a = self.st[self.p + 11];
        self.a = match self.st[self.p + 7] as u8 {
            b'+' => self.w + self.a,
            b'-' => self.w - self.a,
            b'*' => self.w * self.a,
            b'/' => self.w / self.a,
            _ => self.w % self.a,
        };
        self.load();
        self.end_fn();
    }

    // Monitor for errors

    // This routine outputs the item on the stack starting at
    // ST[x]. If the item is not complete, printing stops at
    // ST[S-1] and is followed by '...(Incomplete)'.
    fn item(&mut self, x: usize) {
        print!(" ");
        let (a, h) = (self.a, self.h);
        self.h = 0;
        let mut k = 1;
        loop {
            if self.st[x] == 0 {
                if k == self.s - x {
                    break;
                }
            } else {
                if k == self.st[x] {
                    break;
                }
            }
            self.a = self.st[x + k];
            self.load();
            k += 1;
        }
        if self.st[x] == 0 {
            print!("...\t(Incomplete)");
        }
        self.a = a;
        self.h = h;
    }

    // Monitor entries and effects

    // Unmatched ; in definition string. Treated
    // as <;>
    fn monitor1(&mut self) {
        print!("\nMONITOR: Unmatched semicolon in definition of");
        self.item(self.p + 2);
        print!("\nIf this had been quoted the result would be \n");
        self.copy()
    }

    // Unquoted ~ in argument list in input
    // stream. Treated as <~>
    fn monitor2(&mut self) {
        print!("\nMONITOR: Unquoted tilde in argument list of");
        self.item(self.f + 2);
        print!("\nIf this had been quoted the result would be \n");
        self.copy()
    }

    // Impossible charcter (negative) as argument
    // number. Terminate.
    fn monitor3(&mut self) {
        print!("\nMONITOR: Impossible argument number in definition of");
        self.item(self.p + 2);
        self.monitor11()
    }

    // Not enough arguments supplied in call.
    // Terminate.
    fn monitor4(&mut self) {
        print!("\nMONITOR: No argument ");
        self.h = 0;
        self.load();
        print!(" in call for");
        self.item(self.p + 2);
        self.monitor11()
    }

    // Terminator in impossible place; if C == 0,
    // this is the input stream. Probably
    // machine error: Terminate. If C != 0, this
    // is an argument list. Probably due to a
    // missing semicolon: Final semicolon inserted.
    fn monitor5(&mut self) {
        print!("\nMONITOR: Terminator in");
        if self.c == 0 {
            print!("input stream. Probably machine error.");
            return self.monitor11();
        }
        print!("argument list for");
        self.item(self.f + 2);
        print!("\nProbably due to a semicolon missing from the definition of");
        self.item(self.p + 2);
        print!("\nIf a final semicolon is inserted the result is \n");
        self.c -= 1;
        self.apply()
    }

    // Undefined macro name: Terminate.
    fn monitor7(&mut self) {
        print!("\nMONITOR: Undefined name");
        self.item(self.w);
        self.monitor11()
    }

    // Wrong exit (not C == H == 0). Machine
    // error: Terminate.
    fn monitor8(&mut self) {
        print!("\nMONITOR: Unmatched >. Probably machine error.");
        self.monitor11()
    }

    // Update string too long: Terminate.
    fn monitor9(&mut self) {
        print!("\nMONITOR: Update argument too long for");
        self.item(self.p + 9);
        self.monitor11()
    }

    // Non-digit in argument for BIN. Terminate.
    fn monitor10(&mut self) {
        print!("\nMONITOR: Non-digit in number");
        self.monitor11()
    }

    // General monitor after irremediable
    // errors.
    fn monitor11(&mut self) {
        self.w = 20;
        print!("\nCurrent macros are");
        while self.p != 0 || self.f != 0 {
            let mut w1;
            if self.p > self.f {
                w1 = self.p + 2;
                self.p = self.st[self.p];
                print!("\nAlready entered");
            } else {
                w1 = self.f + 2;
                self.f = self.st[self.f];
                print!("\nNot yet entered");
            }
            for r in 1..self.w {
                self.item(w1);
                if self.st[w1] == 0 {
                    break;
                }
                w1 += self.st[w1];
                if self.st[w1] == self.marker {
                    break;
                }
                if self.w != 1 {
                    print!("\nArg {}\t", r);
                }
            }
            self.w = 1;
        }
        print!("\nEnd of monitor printing");
        self.a = b'Q' as usize;
        self.load();
        if self.p > self.f {
            self.end_fn();
        } else {
            self.start();
        }
    }
}

// These are implementation-dependent functions. They
// convert the `usize` equivalent of a decimal digit read in
// with `read_symbol` to the corresponding number (also of
// type `usize`) and vice versa
fn number(x: usize) -> usize {
    (x as isize - b'0' as isize) as usize
}

fn char_(x: usize) -> usize {
    x + b'0' as usize
}


fn read_symbol(c: &mut usize) {
    let byte = match io::stdin().bytes().next() {
        Some(b) => b,
        None => {
            eprintln!("Unexpected EOF");
            process::exit(1);
        }
    }.unwrap();
    *c = byte as usize;
}

fn write_symbol(c: &usize) {
    assert_eq!(1, io::stdout().write(&[*c as u8]).unwrap());
}

fn main() {
    let mut gpm = GPM::new(10_000);
    gpm.start();
}
