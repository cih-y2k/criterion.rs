use std::fmt::Show;
use std::io::{BufferedReader, Command, PipeStream, Process};
use time;

use routine::Routine;

// A two-way channel to the standard streams of a child process
pub struct Program {
    stdin: PipeStream,
    // NB Don't move the `stdin` field, because it must be dropped first
    _process: Process,
    stderr: PipeStream,
    stdout: BufferedReader<PipeStream>,
}

impl Program {
    pub fn spawn(cmd: &Command) -> Program {
        let mut process = match cmd.spawn() {
            Err(e) => fail!("`{}`: {}", cmd, e),
            Ok(process) => process,
        };

        Program {
            stderr: process.stderr.take().unwrap(),
            stdin: process.stdin.take().unwrap(),
            stdout: BufferedReader::new(process.stdout.take().unwrap()),
            _process: process,
        }
    }

    pub fn send<T: Show>(&mut self, line: T) -> &mut Program {
        match writeln!(self.stdin, "{}", line) {
            Err(e) => fail!("`write into child stdin`: {}", e),
            Ok(_) => self,
        }
    }

    pub fn recv(&mut self) -> String {
        match self.stdout.read_line() {
            Err(e) => {
                println!("stderr:\n{}", self.stderr.read_to_string().unwrap());

                fail!("`read from child stdout`: {}", e);
            },
            Ok(line) => line,
        }
    }
}

impl Routine for Program {
    fn bench<I: Iterator<u64>>(&mut self, mut iters: I) -> Vec<u64> {
        let mut n = 0u;
        for iters in iters {
            self.send(iters);
            n += 1;
        }

        range(0, n).map(|_| {
            let msg = self.recv();
            let msg = msg.as_slice().trim();

            from_str(msg).expect("Couldn't parse program output")
        }).collect()
    }

    fn warm_up(&mut self, how_long_ns: u64) -> (u64, u64) {
        let mut iters = 1;
        let ns_start = time::precise_time_ns();

        loop {
            let elapsed =
                from_str(self.send(iters).recv().as_slice().trim()).
                    expect("Couldn't parse the program output");

            if time::precise_time_ns() - ns_start > how_long_ns {
                return (elapsed, iters);
            }

            iters *= 2;
        }
    }
}