//! Audio output, the same way NIRUSID does it: 16-bit stereo PCM written to
//! a small helper program on a pipe. The first that works of `pw-cat`
//! (PipeWire), `pacat` (PulseAudio or pipewire-pulse) and `aplay` (ALSA) is
//! used, or the `audio` command from the configuration. The pipe is kept
//! small (about 46 ms) so sound effects follow the action, and writing to it
//! paces the synthesiser thread: no timers.
use std::io::Write;
use std::os::fd::AsRawFd;
use std::process::{Child, ChildStdin, Command, Stdio};

pub struct Sink {
    child: Child,
    stdin: ChildStdin,
    #[allow(dead_code)]
    pub rate: u32,
    pub name: String,
}

fn in_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|p| {
        std::env::split_paths(&p).any(|d| {
            let f = d.join(name);
            f.is_file()
                && std::fs::metadata(&f)
                    .map(|m| {
                        use std::os::unix::fs::PermissionsExt;
                        m.permissions().mode() & 0o111 != 0
                    })
                    .unwrap_or(false)
        })
    })
}

/// pw-cat learned raw input (`--raw`) in PipeWire 0.3.5x; older ones need
/// a file format, so they are skipped.
fn pw_cat_has_raw() -> bool {
    static RAW: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *RAW.get_or_init(|| {
        Command::new("pw-cat")
            .arg("--help")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("--raw"))
            .unwrap_or(false)
    })
}

/// The commands to try, in order, for this rate.
pub fn candidates(custom: &str, rate: u32) -> Vec<(String, Command)> {
    let mut out = Vec::new();
    if !custom.trim().is_empty() {
        let mut c = Command::new("sh");
        c.arg("-c").arg(custom.replace("%r", &rate.to_string()));
        out.push(("custom".to_string(), c));
        return out;
    }
    if in_path("pw-cat") && pw_cat_has_raw() {
        let mut c = Command::new("pw-cat");
        c.args(["--playback", "--raw", "--format", "s16", "--channels", "2"])
            .args(["--rate", &rate.to_string()])
            .args(["--latency", "40ms", "--media-role", "Game", "-"]);
        out.push(("pw-cat".to_string(), c));
    }
    if in_path("pacat") {
        let mut c = Command::new("pacat");
        c.args(["--playback", "--raw", "--format=s16le", "--channels=2"])
            .arg(format!("--rate={rate}"))
            .args([
                "--latency-msec=40",
                "--client-name=NIRUX",
                "--stream-name=NIRUX",
            ]);
        out.push(("pacat".to_string(), c));
    }
    if in_path("aplay") {
        let mut c = Command::new("aplay");
        c.args(["-q", "-t", "raw", "-f", "S16_LE", "-c", "2"])
            .args(["-r", &rate.to_string(), "--buffer-time=60000", "-"]);
        out.push(("aplay".to_string(), c));
    }
    out
}

impl Sink {
    /// Start the `skip`th candidate (the earlier ones failed).
    pub fn open(custom: &str, rate: u32, skip: usize) -> Result<Sink, String> {
        let list = candidates(custom, rate);
        if list.is_empty() {
            return Err(
                "no audio output: install PipeWire (pw-cat), pulseaudio-utils (pacat) or alsa-utils (aplay)"
                    .into(),
            );
        }
        let mut last = String::new();
        for (name, mut cmd) in list.into_iter().skip(skip) {
            match cmd
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(mut child) => {
                    let Some(stdin) = child.stdin.take() else {
                        let _ = child.kill();
                        let _ = child.wait();
                        continue;
                    };
                    // About 46 ms of 44.1 kHz stereo in the pipe: sound effects stay snappy.
                    unsafe { libc::fcntl(stdin.as_raw_fd(), libc::F_SETPIPE_SZ, 8192) };
                    return Ok(Sink {
                        child,
                        stdin,
                        rate,
                        name,
                    });
                }
                Err(e) => last = format!("{name}: {e}"),
            }
        }
        Err(format!("no audio output could be started ({last})"))
    }

    pub fn write(&mut self, pcm: &[i16]) -> std::io::Result<()> {
        // SAFETY: i16 has no padding; the slice covers exactly the samples.
        let bytes = unsafe { std::slice::from_raw_parts(pcm.as_ptr() as *const u8, pcm.len() * 2) };
        self.stdin.write_all(bytes)
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
