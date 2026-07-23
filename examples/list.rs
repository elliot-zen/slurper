use slurper::backend::{self, AgentKind};

fn main() {
    for kind in AgentKind::ALL {
        match backend::backend(kind).list() {
            Ok(sessions) => {
                println!("== {} ({} sessions)", kind.name(), sessions.len());
                for s in sessions.iter().take(3) {
                    println!(
                        "  {} | {} | {} | {}",
                        &s.id[..s.id.len().min(20)],
                        s.title,
                        s.cwd,
                        s.updated_ms
                    );
                }
            }
            Err(e) => println!("== {} ERROR: {e:#}", kind.name()),
        }
    }
}
