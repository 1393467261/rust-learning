use std::sync::{Mutex, Arc};

pub enum State {
    Follower,
    Candidate,
    Leader,
    Dead
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            State::Follower => "Follower".to_string(),
            State::Candidate => "Candidate".to_string(),
            State::Leader => "Leader".to_string(),
            State::Dead => "Dead".to_string(),
        }
    }
}

pub struct  ConsensusModule {
    mu: Mutex<()>,
    id: usize,
    peer_ids: Vec<usize>,
    server: Arc<Mutex<Server>>,
    current_term: usize,
    voted_for: usize,
    log: Vec<LogEntry>,
    state: State,
    
}

pub struct LogEntry {

}

pub struct Server {

}

#[test]
fn test_state_to_string() {
    assert_eq!(State::Leader.to_string(), "Leader".to_string());
}