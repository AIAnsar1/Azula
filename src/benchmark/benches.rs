use std::time::Instant;



#[derive(Debug)]
pub struct Benchmark {
    named_timers: Vec<NamedTimer>,
}


#[derive(Debug)]
pub struct NamedTimer {
    name: &'static str,
    start: Option<Instant>,
    end: Option<Instant>,
}


impl Benchmark {
    pub fn init() -> Self {
        Self {
            named_timers: Vec::new(),
        }
    }

    pub fn push(&mut self, timer: NamedTimer) {
        self.named_timers.push(timer)
    }

    pub fn summary(&self) -> String {
        let mut summary = String::from("\n Azula Benchmark Summary!\n");

        for timer in &self.named_timers {
            if timer.start.is_some() && timer.end.is_some() {
                let runtime = timer.end.unwrap().saturating_duration_since(timer.start.unwrap()).as_secs_f32();
                summary.push_str(&format!("\n{0: <10} | {1: <10}s", timer.name, runtime));
            }
        }
        summary
    }
}



impl NamedTimer {
    pub fn start(name: &'static str) -> Self {
        Self {
            name, start: Some(Instant::now()), end: None,
        }
    }
    pub fn end(&mut self) {
        self.end = Some(Instant::now());
    }
}
