// struct, enum
struct Server {
    // Persistent state on all servers
    current_term: usize,
    vote_for: usize,
    log: Vec<Log>,

    // Volatile state on all servers
    commit_index: usize,
    last_applied: usize,
}

struct Leader {
    server: Server,
    // Volatile state on leaders
    next_index: Vec<usize>,
    match_index: Vec<usize>,
}

struct Candidate {
    server: Server,
}

struct Follower {
    server: Server,
}

struct AppendEntries {
    term: usize,
    leader_id: usize,
    pre_log_index: usize,
    pre_log_term: usize,
    entries: Vec<Log>,
    leader_commit: usize,
}

struct AppendEntriesResponse {
    term: usize,
    success: bool,
}

struct RequestVote {
    term: usize,
    candidated_id: usize,
    last_log_index: usize,
    last_log_term: usize,
}

struct RequestVoteResponse {
    term: usize,
    vote_granted: bool,
}

struct Log {}

// trait
trait Participator {
    fn start_election(&self);
}

impl Participator for Leader {
    fn start_election(&self) {
        println!("leader start election")
    }
}

impl Participator for Candidate {
    fn start_election(&self) {
        println!("Candidate start election")
    }
}

impl Participator for Follower {
    fn start_election(&self) {
        println!("Follower start election")
    }
}

// impl struct
impl Server {
    fn new() -> Server {
        Server 
        { 
            current_term: 0, 
            vote_for: 0, 
            log: vec![], 
            commit_index: 0, 
            last_applied: 0, 
        }
    }
}

impl Leader {
    fn new() -> Leader {
        Leader {
            server: Server::new(),
            next_index: vec![],
            match_index: vec![],
        }
    }
}

impl Candidate {
    fn new() -> Candidate {
        Candidate {
            server: Server::new(),
        }
    }
}

impl Follower {
    fn new() -> Follower {
        Follower {
            server: Server::new(),
        }
    }
}

#[test]
fn test_trait() {
    let server = Leader::new();
    server.start_election();
}