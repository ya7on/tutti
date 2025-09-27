use std::time::SystemTime;

#[derive(Debug)]
pub enum Status {
    Starting,
    Running,
    Exited(i32),
    Failed,
}

#[derive(Debug)]
pub struct ServiceState {
    pub name: String,
    pub pid: Option<u32>,
    pub status: Status,
    pub started_at: Option<SystemTime>,
}
